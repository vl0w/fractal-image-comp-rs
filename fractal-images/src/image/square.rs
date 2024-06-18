use std::ops::Deref;
use thiserror::Error;
use crate::image::Image;

/// Represents a square image, i.e. an image whose [size](Size) is a square.
///
/// Wraps an image object with the additional constraint that the image must be square.
/// This struct does not provide any additional logic, but rather validates that the underlying image conforms to the square shape.
///
/// # Usage
///
/// This struct is useful to ensure compile-time guarantees that the contained image is a square.
///
/// # Examples
/// ```rust
/// use fractal_image::image::{Image, FakeImage, Square};
///
/// let image: Square<FakeImage> = FakeImage::squared(4);
///
/// assert!(image.get_size().is_squared());
/// assert_eq!(image.get_width(), image.get_height());
/// ```
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Square<I> (I);

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error(
    "The provided image is not a square, height = {} != {} = width", .0.get_height(), .0.get_width()
)]
pub struct NotSquareError<I: Image>(I);

impl<I> Square<I>
where
    I: Image,
{
    pub fn new(image: I) -> Result<Self, NotSquareError<I>> {
        if image.get_size().is_squared() {
            Ok(Square(image))
        } else {
            Err(NotSquareError(image))
        }
    }
}

impl<I> Deref for Square<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[cfg(test)]
mod tests {
    use crate::image::fake::FakeImage;
    use crate::image::Size;
    use crate::size;
    use super::*;

    #[test]
    fn squared_image_test_success() {
        let image = FakeImage::new(size!(w=100,h=100));
        let squared = Square::new(image);
        assert!(squared.is_ok());
    }

    #[test]
    fn squared_image_test_failure() {
        let image = FakeImage::new(size!(w=100,h=101));
        let squared = Square::new(image);
        assert!(squared.is_err());
        assert_eq!(squared.unwrap_err(), NotSquareError(FakeImage::new(size!(w=100,h=101))));
    }
}