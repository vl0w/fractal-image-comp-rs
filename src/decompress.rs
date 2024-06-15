use crate::image::block::SquaredBlock;
use crate::image::downscale::IntoDownscaled;
use crate::image::owned::OwnedImage;
use crate::image::{Image, IterablePixels, MutableImage, Size};
use crate::model::{Compressed, Transformation};
use crate::preprocessing::SafeableImage;
use image::ImageFormat;
use std::path::Path;
use std::sync::Arc;
use tracing::instrument;

#[instrument(level = "debug", skip(compressed))]
pub fn decompress(compressed: Compressed, iterations: u8) -> OwnedImage {
    let mut image = OwnedImage::random(Size::new(compressed.width, compressed.height));

    for iteration in 0..iterations {
        let previous_pass = Arc::new(image.clone());
        for transformation in compressed.transformations.iter() {
            transformation.apply_to(previous_pass.clone(), &mut image);
        }
        let filename = format!("decompressed_{}.png", iteration);
        image.save_image(Path::new(&filename), ImageFormat::Png)
    }

    image
}

impl Transformation {
    fn apply_to(&self, previous_pass: Arc<OwnedImage>, image: &mut OwnedImage) {
        let domain_block = SquaredBlock {
            image: previous_pass,
            origin: self.domain.origin,
            size: self.domain.block_size,
        };

        let domain_block = domain_block.downscale_2x2();
        let indices = self.range.indices(image.get_width(), image.get_height());

        for ((_, coords), db_pixel) in indices.zip(domain_block.pixels()) {
            let new_pixel_value: f64 = db_pixel as f64 * self.saturation + self.brightness as f64;
            let new_pixel_value = new_pixel_value.min(255.0).max(0.0) as u8;
            image.set_pixel(coords.x, coords.y, new_pixel_value);
        }
    }
}
