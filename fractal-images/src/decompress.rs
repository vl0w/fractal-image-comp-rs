use std::sync::Arc;

use tracing::instrument;

use crate::image::{Image, IterablePixels, MutableImage};
use crate::image::block::SquaredBlock;
use crate::image::downscale::IntoDownscaled;
use crate::image::owned::OwnedImage;
use crate::image::rotate::IntoRotated;
use crate::model::{Compressed, Transformation};

#[derive(Debug)]
pub struct Options {
    pub iterations: u8,
    pub keep_each_iteration: bool,
}

pub struct Decompressed {
    pub image: OwnedImage,
    pub iterations: Option<Vec<OwnedImage>>,
}

#[instrument(level = "debug", skip(compressed))]
pub fn decompress(compressed: Compressed, options: Options) -> Decompressed {
    let mut image = OwnedImage::random(compressed.size);
    let mut image_per_iteration: Option<Vec<OwnedImage>> = match options.keep_each_iteration {
        false => None,
        true => Some(vec![]),
    };

    for _ in 0..options.iterations {
        let previous_pass = Arc::new(image.clone());
        for transformation in compressed.transformations.iter() {
            transformation.apply_to(previous_pass.clone(), &mut image);
        }

        match image_per_iteration.as_mut() {
            None => (),
            Some(it) => it.push(image.clone()),
        }
    }

    Decompressed {
        image,
        iterations: image_per_iteration,
    }
}

impl Transformation {
    fn apply_to(&self, previous_pass: Arc<OwnedImage>, image: &mut OwnedImage) {
        let domain_block = SquaredBlock {
            image: previous_pass,
            origin: self.domain.origin,
            size: self.domain.block_size,
        };

        let domain_block = domain_block.downscale_2x2().rot(self.rotation);
        let indices = self.range.indices(image.get_width(), image.get_height());

        for ((_, coords), db_pixel) in indices.zip(domain_block.pixels()) {
            let new_pixel_value: f64 = db_pixel as f64 * self.saturation + self.brightness as f64;
            let new_pixel_value = new_pixel_value.min(255.0).max(0.0) as u8;
            image.set_pixel(coords.x, coords.y, new_pixel_value);
        }
    }
}
