use std::cmp::min;
use std::path::{Path, PathBuf};
use image::{DynamicImage, GrayImage, ImageFormat};
use image::imageops::FilterType;
use crate::image::{Image, IterablePixels, Pixel};

pub struct SquaredGrayscaleImage {
    pixels: Vec<u8>,
    size: u32,
}

impl SquaredGrayscaleImage {
    pub fn read_from(path: &Path) -> Self {
        let image = image::open(path).expect(format!("Could not load image: {:?}", path).as_str());
        let size = min(image.width(), image.height());
        let image = image.resize(size, size, FilterType::Gaussian);
        let image = image.to_rgb8();
        let grayscale = image.pixels().map(|pixel| {
            let red = pixel.0[0];
            let green = pixel.0[1];
            let blue = pixel.0[2];
            let ntsc_grayscale = 299 * red as u32 + 587 * green as u32 + 114 * blue as u32;
            let ntsc = ntsc_grayscale / 1000;
            ntsc as u8
        }).collect::<Vec<_>>();

        Self {
            pixels: grayscale,
            size,
        }
    }
}

impl Image for SquaredGrayscaleImage {
    fn get_width(&self) -> u32 {
        self.size
    }

    fn get_height(&self) -> u32 {
        self.size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let index = self.get_width() * y + x;
        self.pixels[index as usize]
    }

}

// impl IterablePixels for SquaredGrayscaleImage {
//     fn pixels(&self) -> impl Iterator<Item=&Pixel> {
//         self.pixels.iter()
//     }
// }

pub trait AsDynamicImage {
    fn as_dynamic_image(&self) -> DynamicImage;
}

impl<T> AsDynamicImage for T where T: Image + IterablePixels {
    fn as_dynamic_image(&self) -> DynamicImage {
        let pixels: Vec<_> =  self.pixels().collect();
        print!("{}",pixels.len());
        let image = GrayImage::from_raw(self.get_width(), self.get_height(), pixels).expect("Unable to convert to GrayImage");
        DynamicImage::ImageLuma8(image)
    }
}

pub trait SafeableImage {
    fn save_image(&self, path: &Path, format: ImageFormat);

    fn save_image_as_png(&self, path: &Path) {
        self.save_image(path, ImageFormat::Png)
    }
}

impl<T> SafeableImage for T where T: AsDynamicImage {
    fn save_image(&self, path: &Path, format: ImageFormat) {
        let image = self.as_dynamic_image();
        image.save_with_format(path, format).expect(format!("Could not save image to {:?}", path).as_str());
    }
}