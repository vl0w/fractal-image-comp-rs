use std::ops::Deref;
use thiserror::Error;
use crate::image::Image;

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
/// use fractal_image::image::{Image, fake::FakeImage, power_of_two::PowerOfTwo, Size};
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PowerOfTwo<I> (I);

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error(
    "The provided image's width or height is not a power of two, height = {}, {} = width", .0.get_height(), .0.get_width()
)]
pub struct NoPowerOfTwo<I: Image>(I);

impl<I> PowerOfTwo<I>
where
    I: Image,
{
    pub fn new(image: I) -> Result<Self, NoPowerOfTwo<I>> {
        if !is_power_of_two(image.get_width()) || !is_power_of_two(image.get_height()) {
            Err(NoPowerOfTwo(image))
        } else {
            Ok(Self(image))
        }
    }
}

impl<I> Deref for PowerOfTwo<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target { &self.0 }
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