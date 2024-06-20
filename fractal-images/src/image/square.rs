use std::sync::Arc;

use thiserror::Error;

use crate::image::{Image, Pixel, Size};

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
pub struct Square<I> (Arc<I>);

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error(
    "The provided image is not a square, height = {} != {} = width", .0.get_height(), .0.get_width()
)]
pub struct NotSquareError<I: Image>(Arc<I>);

impl<I> Square<I>
where
    I: Image,
{
    pub fn new(image: I) -> Result<Self, NotSquareError<I>> {
        Self::new_arc(Arc::new(image))
    }

    pub fn new_arc(image: Arc<I>) -> Result<Self, NotSquareError<I>> {
        if image.get_size().is_squared() {
            Ok(Self(image))
        } else {
            Err(NotSquareError(image))
        }
    }

    pub fn as_inner(&self) -> Arc<I> {
        self.0.clone()
    }

    pub fn into_inner(self) -> Arc<I> {
        self.0
    }
}

impl<I> Image for Square<I>
where
    I: Image,
{
    fn get_size(&self) -> Size {
        self.0.get_size()
    }
    fn get_height(&self) -> u32 {
        self.0.get_height()
    }
    fn get_width(&self) -> u32 {
        self.0.get_width()
    }
    fn pixel(&self, x: u32, y: u32) -> Pixel {
        self.0.pixel(x, y)
    }
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
        assert_eq!(squared.unwrap_err(), NotSquareError(Arc::new(FakeImage::new(size!(w=100,h=101)))));
    }
}