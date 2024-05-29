use tracing::trace;
use crate::image::{Image, IterablePixels};

pub mod quadtree;

#[derive(Debug, Clone, Copy)]
struct Mapping {
    error: f64,
    brightness: i16,
    saturation: f64,
}

impl Mapping {
    fn compute<A, B>(domain: &A, range: &B) -> Self
        where A: Image + IterablePixels, B: Image + IterablePixels {
        assert_eq!(domain.get_height(), range.get_height());
        assert_eq!(domain.get_width(), range.get_width());

        let n: f64 = (domain.get_width() * domain.get_height()) as f64; // amount of pixels

        let a = domain.pixels().map(|x| x as f64);
        let b = range.pixels().map(|x| x as f64);
        let a_times_b_sum: f64 = a.zip(b).map(|(a, b)| a * b).sum();

        let a = domain.pixels().map(|x| x as f64);
        let b = range.pixels().map(|x| x as f64);
        let a_squared_sum: f64 = a.map(|x| x * x).sum();
        let b_squared_sum: f64 = b.map(|x| x * x).sum();

        let a = domain.pixels().map(|x| x as f64);
        let b = range.pixels().map(|x| x as f64);
        let a_sum: f64 = a.sum();
        let b_sum: f64 = b.sum();
        let a_sum_squared = a_sum.powi(2);

        // Compute s (saturation)
        let denominator = n * a_squared_sum - a_sum_squared;
        let s = match denominator {
            0.0 => 0.0,
            _ => (n * a_times_b_sum - a_sum * b_sum) / denominator
        };

        // Compute o (brightness)
        let o = match denominator {
            0.0 => b_sum / n,
            _ => (b_sum - s * a_sum) / n
        };

        // Squared error
        let r = (b_squared_sum + s * (s * a_squared_sum - 2.0 * a_times_b_sum + 2.0 * o * a_sum) + o * (n * o - 2.0 * b_sum)) / n;
        let rms_error = r.sqrt();

        trace!("saturation = {}", s);
        trace!("brightness = {}", o);
        trace!("RMS error = {}", rms_error);
        Self {
            error: rms_error,
            brightness: (o as i16).min(256).max(0), // TODO: Weird right?
            saturation: s,
        }
    }
}