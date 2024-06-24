use std::cmp::max;
use thiserror::Error;
use crate::image::{Image, IterablePixels, Size};

#[derive(Error, Debug, Clone, Copy, Eq, PartialEq)]
#[error("Can not compare images with different sizes ({} != {})", 0, 1)]
pub struct ImageSizeMismatch(Size, Size);

/// Computes the [MSE](https://en.wikipedia.org/wiki/Mean_squared_error) metric of two images.
pub fn mse<A: Image + IterablePixels, B: Image + IterablePixels>(first: &A, second: &B) -> Result<f64, ImageSizeMismatch> {
    if first.get_size() != second.get_size() {
        return Err(ImageSizeMismatch(first.get_size(), second.get_size()));
    }

    let area = first.get_size().area();

    let sum: f64 = first.pixels().zip(second.pixels())
        .map(|(px_a, px_b)| (px_a as i64 - px_b as i64).pow(2) as f64)
        .sum();

    Ok(sum / area as f64)
}

/// Computes the [PSNR](https://en.wikipedia.org/wiki/Peak_signal-to-noise_ratio) metric of two images.
pub fn psnr<A: Image + IterablePixels, B: Image + IterablePixels>(first: &A, second: &B) -> Result<f64, ImageSizeMismatch> {
    let mse = mse(first, second)?;
    let max_a = first.pixels().max().unwrap_or(0);
    let max_b = second.pixels().max().unwrap_or(0);
    let max = max(max_a, max_b) as f64;

    Ok(20f64 * max.log10() - 10f64 * mse.log10())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mse {
        use fluid::prelude::ShouldExtension;
        use crate::image::FakeImage;
        use super::*;

        #[test]
        fn mse_for_images_with_different_sizes_returns_error() {
            let first = FakeImage::squared(4);
            let second = FakeImage::squared(5);
            let result = mse(
                &first,
                &second,
            );

            result.should().be_an_error()
                .because("two images with inequal sizes are not comparable");
        }
    }

    mod psnr {
        use fluid::prelude::ShouldExtension;
        use crate::image::FakeImage;
        use super::*;

        #[test]
        fn psnr_for_images_with_different_sizes_returns_error() {
            let first = FakeImage::squared(4);
            let second = FakeImage::squared(5);
            let result = psnr(
                &first,
                &second,
            );

            result.should().be_an_error()
                .because("two images with inequal sizes are not comparable");
        }

        #[test]
        fn psnr_for_same_images_returns_infinity() {
            let first = FakeImage::squared(5);
            let second = FakeImage::squared(5);
            let result = psnr(
                &first,
                &second,
            );

            result.should().be_ok();
            result.should().be_equal_to(Ok(f64::INFINITY)).because("two equal images have an infinity PSNR");
        }
    }
}