use crate::image::iter::PixelIterator;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size};
use std::sync::Arc;
pub use conversion::*;

pub struct Downscaled2x2<I> {
    image: Arc<I>,
}

impl<I> Clone for Downscaled2x2<I> {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
        }
    }
}

impl<I: Image> Downscaled2x2<I> {
    pub fn inner(&self) -> Arc<I> {
        self.image.clone()
    }
}

impl<I: Image> Image for Downscaled2x2<I> {
    fn get_size(&self) -> Size {
        self.image.get_size() / 2
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let sum = self.image.pixel(2 * x, 2 * y) as u32
            + self.image.pixel(2 * x + 1, 2 * y) as u32
            + self.image.pixel(2 * x, 2 * y + 1) as u32
            + self.image.pixel(2 * x + 1, 2 * y + 1) as u32;
        (0.25 * sum as f64) as Pixel
    }
}

mod conversion {
    use std::sync::Arc;
    use crate::image::{Downscaled2x2, Image, Square, SquaredBlock};

    pub trait IntoDownscaled<I>
    where
        I: Image,
    {
        type Target;
        
        fn downscale_2x2(self) -> Downscaled2x2<Self::Target>;
    }

    impl<I> IntoDownscaled<I> for &Square<I>
    where
        I: Image,
    {
        type Target = I;
        fn downscale_2x2(self) -> Downscaled2x2<Self::Target> {
            Downscaled2x2 {
                image: self.as_inner(),
            }
        }
    }

    impl<I> IntoDownscaled<I> for &SquaredBlock<I>
    where
        I: Image,
    {
        type Target = SquaredBlock<I>;
        fn downscale_2x2(self) -> Downscaled2x2<Self::Target> {
            Downscaled2x2 {
                image: Arc::new(self.clone()),
            }
        }
    }
}

impl<I: Image> IterablePixels for Downscaled2x2<I> {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        PixelIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::fake::FakeImage;

    #[test]
    fn downscaled_size() {
        let image = FakeImage::squared(16).downscale_2x2();
        assert_eq!(image.get_width(), 8);
        assert_eq!(image.get_height(), 8);
    }

    #[test]
    fn groups_2x2_pixels_of_original_image() {
        // Original image
        // 0  1  2  3
        // 4  5  6  7
        // 8  9  10 11
        // 12 13 14 15

        let image = FakeImage::squared(4).downscale_2x2();
        assert_eq!(image.pixel(0, 0), (1 + 4 + 5) / 4);
        assert_eq!(image.pixel(1, 0), (2 + 3 + 6 + 7) / 4);
        assert_eq!(image.pixel(0, 1), (8 + 9 + 12 + 13) / 4);
        assert_eq!(image.pixel(1, 1), (10 + 11 + 14 + 15) / 4);
    }
    

    #[test]
    #[should_panic]
    fn overflow_x() {
        let image = FakeImage::squared(4).downscale_2x2();
        image.pixel(2, 0);
    }

    #[test]
    #[should_panic]
    fn overflow_y() {
        let image = FakeImage::squared(4).downscale_2x2();
        image.pixel(0, 2);
    }
}
