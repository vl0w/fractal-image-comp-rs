use crate::image::{Coords, Image, IterablePixels, Pixel};
use crate::image::iter::PixelIterator;


pub trait IntoDownscaled<'a, I> where I: Image + 'a {
    fn downscale_2x2(&'a self) -> Downscaled2x2<'a, I>;
}

impl<'a, I> IntoDownscaled<'a, I> for I where I: Image + 'a {
    fn downscale_2x2(&'a self) -> Downscaled2x2<'a, I> {
        Downscaled2x2 {
            image: self
        }
    }
}

pub struct Downscaled2x2<'a, I: Image> {
    image: &'a I,
}

impl<'a, I: Image> Downscaled2x2<'a, I> {
    pub fn inner(&self) -> &'a I {
        self.image
    }
}

impl<'a, I: Image> Image for Downscaled2x2<'a, I> {
    fn get_width(&self) -> u32 {
        self.image.get_width() / 2
    }

    fn get_height(&self) -> u32 {
        self.image.get_height() / 2
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let sum = self.image.pixel(2 * x, 2 * y) as u32 +
            self.image.pixel(2 * x + 1, 2 * y) as u32 +
            self.image.pixel(2 * x, 2 * y + 1) as u32 +
            self.image.pixel(2 * x + 1, 2 * y + 1) as u32;
        (0.25 * sum as f64) as Pixel
    }
}

impl<'a, I: Image> IterablePixels for Downscaled2x2<'a, I> {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        PixelIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils::FakeImage;
    use super::*;

    #[test]
    fn downscaled_size() {
        let image = FakeImage::new(16, 16);
        let scaled = image.downscale_2x2();
        assert_eq!(scaled.get_width(), 8);
        assert_eq!(scaled.get_height(), 8);
    }

    #[test]
    fn groups_2x2_pixels_of_original_image() {
        // Original image
        // 0  1  2  3
        // 4  5  6  7
        // 8  9  10 11
        // 12 13 14 15

        let image = FakeImage::new(4, 4);
        let scaled = image.downscale_2x2();
        assert_eq!(scaled.pixel(0, 0), (1 + 4 + 5) / 4);
        assert_eq!(scaled.pixel(1, 0), (2 + 3 + 6 + 7) / 4);
        assert_eq!(scaled.pixel(0, 1), (8 + 9 + 12 + 13) / 4);
        assert_eq!(scaled.pixel(1, 1), (10 + 11 + 14 + 15) / 4);
    }

    #[test]
    #[should_panic]
    fn overflow_x() {
        let image = FakeImage::new(4, 4);
        let scaled = image.downscale_2x2();
        scaled.pixel(2, 0);
    }

    #[test]
    #[should_panic]
    fn overflow_y() {
        let image = FakeImage::new(4, 4);
        let scaled = image.downscale_2x2();
        scaled.pixel(0, 2);
    }
}

