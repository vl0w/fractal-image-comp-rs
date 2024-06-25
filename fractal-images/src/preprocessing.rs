use crate::image::{Image, Pixel, PowerOfTwo, Size, Square};
use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, ImageFormat};
use std::cmp::min;
use std::path::Path;
use tracing::debug;

#[derive(Debug)]
pub struct SquaredGrayscaleImage {
    pixels: Vec<u8>,
    size: Size,
}

impl SquaredGrayscaleImage {
    pub fn read_from(path: &Path) -> PowerOfTwo<Square<Self>> {
        let image = image::open(path).unwrap_or_else(|_| panic!("Could not load image: {:?}", path));
        let size = min(image.width(), image.height());

        // Ensure size is a multiple of 2
        let size = (size.ilog2() as f32).exp2() as u32;

        let image = image.resize(size, size, FilterType::Gaussian);
        let image = image.to_rgb8();
        let grayscale = image
            .pixels()
            .map(|pixel| {
                let red = pixel.0[0];
                let green = pixel.0[1];
                let blue = pixel.0[2];
                let ntsc_grayscale = 299 * red as u32 + 587 * green as u32 + 114 * blue as u32;
                let ntsc = ntsc_grayscale / 1000;
                ntsc as u8
            })
            .collect::<Vec<_>>();

        let image = Square::new(Self {
            pixels: grayscale,
            size: Size::squared(size),
        }).expect("Unable to create a square image");

        PowerOfTwo::new(image).expect("Unable to downscale image to a power of two")
    }
}

impl Image for SquaredGrayscaleImage {
    fn get_size(&self) -> Size {
        self.size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let index = self.get_width() * y + x;
        self.pixels[index as usize]
    }
}

pub trait AsDynamicImage {
    fn as_dynamic_image(&self) -> DynamicImage;
}

impl<T> AsDynamicImage for T
where
    T: Image,
{
    fn as_dynamic_image(&self) -> DynamicImage {
        debug!("Converting image to dynamic image");
        let pixels: Vec<_> = self.pixels().collect();
        let image = GrayImage::from_raw(self.get_width(), self.get_height(), pixels)
            .expect("Unable to convert to GrayImage");
        DynamicImage::ImageLuma8(image)
    }
}

pub trait SafeableImage {
    fn save_image(&self, path: &Path, format: ImageFormat);

    fn save_image_as_png<T: AsRef<Path>>(&self, path: T) {
        self.save_image(path.as_ref(), ImageFormat::Png)
    }
}

impl<T> SafeableImage for T
where
    T: AsDynamicImage,
{
    fn save_image(&self, path: &Path, format: ImageFormat) {
        let image = self.as_dynamic_image();
        image
            .save_with_format(path, format)
            .unwrap_or_else(|_| panic!("Could not save image to {:?}", path));
    }
}
