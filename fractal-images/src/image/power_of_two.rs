use std::sync::Arc;
use derive_more::Display;
use thiserror::Error;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size};

/// Represents an image with dimensions that are powers of two.
///
/// Wraps an image object with the additional constraint that both the width and
/// the height of the image are powers of two. This struct does not provide
/// any additional logic, but rather validates that the underlying image
/// dimensions conform to the power of two requirement.
///
/// # Usage
///
/// This struct is useful to ensure compile-time guarantees that the
/// dimensions of the contained image are powers of two.
///
/// # Examples
/// ```rust
/// use fractal_image::image::{Image, FakeImage, PowerOfTwo, Size};
/// use fractal_image::size;
///
/// assert!(PowerOfTwo::new(
///     FakeImage::new(size!(width=4, height=16))
/// ).is_ok());
/// assert!(PowerOfTwo::new(
///     FakeImage::new(size!(width=3, height=4))
/// ).is_err());
/// assert!(PowerOfTwo::new(
///     FakeImage::new(size!(width=32, height=10))
/// ).is_err());
/// assert!(PowerOfTwo::new(
///     FakeImage::new(size!(width=13, height=21))
/// ).is_err());
/// ```
///
#[derive(Clone, Debug, Eq, PartialEq, Display)]
pub struct PowerOfTwo<I> (Arc<I>);

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error(
    "The provided image's width or height is not a power of two, height = {}, width = {}", .0.get_height(), .0.get_width()
)]
pub struct NoPowerOfTwo(Size);

impl<I> PowerOfTwo<I>
where
    I: Image,
{
    pub fn new(image: I) -> Result<Self, NoPowerOfTwo> {
        Self::new_arc(Arc::new(image))
    }

    pub fn new_arc(image: Arc<I>) -> Result<Self, NoPowerOfTwo> {
        if !is_power_of_two(image.get_width()) || !is_power_of_two(image.get_height()) {
            Err(NoPowerOfTwo(image.get_size()))
        } else {
            Ok(Self(image))
        }
    }

    pub fn as_inner(&self) -> Arc<I> {
        self.0.clone()
    }

    pub fn into_inner(self) -> Arc<I> {
        self.0
    }
}

impl<I> Image for PowerOfTwo<I>
where
    I: Image,
{
    fn get_size(&self) -> Size {
        self.0.get_size()
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        self.0.pixel(x, y)
    }
}

impl<I> IterablePixels for PowerOfTwo<I> where I: IterablePixels {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        self.0.pixels_enumerated()
    }
}

fn is_power_of_two(val: u32) -> bool {
    val != 0 && (val & (val - 1)) == 0
}

#[cfg(test)]
mod tests {
    use crate::image::fake::FakeImage;
    use crate::image::Size;
    use crate::size;
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(!is_power_of_two(3));
        assert!(is_power_of_two(4));
        assert!(!is_power_of_two(5));
        assert!(is_power_of_two(8));
        assert!(is_power_of_two(16));
        assert!(is_power_of_two(32));
    }

    #[test]
    fn test_power_of_two_success() {
        assert!(
            PowerOfTwo::new(FakeImage::new(size!(w=4,h=4))).is_ok()
        );
    }

    #[test]
    fn test_power_of_two_fail() {
        assert!(PowerOfTwo::new(FakeImage::new(
            size!(w=3,h=4)
        )).is_err());
        assert!(PowerOfTwo::new(FakeImage::new(
            size!(w=4,h=3)
        )).is_err());
        assert!(PowerOfTwo::new(FakeImage::new(
            size!(w=3,h=3)
        )).is_err());
    }
}