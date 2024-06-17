use crate::image::{Image, IterablePixels};
use tracing::trace;

pub mod quadtree;

#[derive(Debug, Clone, Copy)]
struct Mapping {
    error: f64,
    brightness: i16,
    saturation: f64,
}

impl Mapping {
    fn compute<A, B>(domain: &A, range: &B) -> Option<Self>
    where
        A: Image + IterablePixels,
        B: Image + IterablePixels,
    {
        assert_eq!(domain.get_height(), range.get_height());
        assert_eq!(domain.get_width(), range.get_width());

        let n: f64 = (domain.get_width() * domain.get_height()) as f64; // amount of pixels

        let (mut domain_times_range_sum, mut domain_squared_sum, mut range_squared_sum, mut domain_sum, mut range_sum) =
            (0.0, 0.0, 0.0, 0.0, 0.0);
        for (dp, rp) in domain.pixels().zip(range.pixels()) {
            let dp = dp as f64;
            let rp = rp as f64;
            domain_times_range_sum += dp * rp;
            domain_squared_sum += dp * dp;
            range_squared_sum += rp * rp;
            domain_sum += dp;
            range_sum += rp;
        }
        let domain_sum_squared = domain_sum * domain_sum;

        // Compute s (saturation)
        let denominator = n * domain_squared_sum - domain_sum_squared;
        let saturation = match denominator {
            0.0 => 0.0,
            _ => (n * domain_times_range_sum - domain_sum * range_sum) / denominator,
        };

        // Compute o (brightness)
        let brightness = match denominator {
            0.0 => range_sum / n,
            _ => (range_sum - saturation * domain_sum) / n,
        }.clamp(0.0, 255.0);

        // Squared error
        let error = (range_squared_sum
            + saturation * (saturation * domain_squared_sum - 2.0 * domain_times_range_sum + 2.0 * brightness * domain_sum)
            + brightness * (n * brightness - 2.0 * range_sum))
            / n;

        let rms_error = if saturation.abs() > 1.0 {
            return None;
        } else {
            error.sqrt()
        };

        trace!("saturation = {}", saturation);
        trace!("brightness = {}", brightness);
        trace!("RMS error = {}", rms_error);

        Some(Self {
            error: rms_error,
            brightness: brightness as i16,
            saturation,
        })
    }
}