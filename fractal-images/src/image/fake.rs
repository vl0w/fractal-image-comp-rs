use crate::image::{Image, PowerOfTwo};
use crate::image::Pixel;
use crate::image::Size;
use crate::image::square::Square;

/// An image, whose pixel values
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FakeImage {
    size: Size,
}

impl Image for FakeImage {
    fn get_size(&self) -> Size {
        self.size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.get_width());
        assert!(y < self.get_height());
        (y * self.get_width() + x) as u8
    }
}

impl FakeImage {
    pub fn new(size: Size) -> Self {
        Self { size }
    }

    /// Returns an image which is a square.
    pub fn squared(size: u32) -> Square<Self> {
        Square::new(Self::new(Size::squared(size))).unwrap()
    }

    /// Returns an image which is a square and whose size is a power of two.
    pub fn squared_power_of_two(exponent: u16) -> PowerOfTwo<Square<Self>> {
        let size = 2u32.pow(exponent as u32);
        let image = Self::squared(size);
        PowerOfTwo::new(image).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let img = FakeImage::new(Size::new(10, 20));
        assert_eq!(img.get_size(), Size::new(10, 20));
    }

    #[test]
    fn test_pixel() {
        let img = FakeImage::new(Size::new(10, 10));
        assert_eq!(img.pixel(0, 0), 0);
        assert_eq!(img.pixel(5, 5), (5 * 10 + 5) as u8);
    }

    #[test]
    #[should_panic]
    fn test_pixel_out_of_bound() {
        let img = FakeImage::new(Size::new(10, 10));

        img.pixel(11, 11);
    }

    #[test]
    fn test_squared() {
        let size = 10;
        let img = FakeImage::squared(size);
        assert_eq!(img.get_size(), Size::new(size, size));
    }
}
