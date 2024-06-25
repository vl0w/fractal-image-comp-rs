use rand::{Rng, SeedableRng};

use crate::image::{Image, MutableImage, Pixel, Size};

/// A type which stores pixel values in a `Vec`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedImage {
    size: Size,
    data: Vec<u8>,
}

impl OwnedImage {
    pub fn random(size: Size) -> Self {
        Self::random_with_seed(size, size.area() as u64)
    }
    
    pub fn random_with_seed(size: Size, seed: u64) -> Self {
        let mut data = Vec::with_capacity((size.area()) as usize);
        let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
        for _ in 0..size.area() {
            data.push(rng.gen_range(0..256) as Pixel);
        }

        Self { size, data }
    }
}

impl Image for OwnedImage {
    fn get_size(&self) -> Size {
        self.size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.get_width());
        assert!(y < self.get_height());
        let idx = (y * self.get_width() + x) as usize;
        self.data[idx]
    }
}

impl MutableImage for OwnedImage {
    fn set_pixel(&mut self, x: u32, y: u32, value: Pixel) {
        assert!(x < self.get_width());
        assert!(y < self.get_height());
        let idx = (y * self.get_width() + x) as usize;
        self.data[idx] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_random_owned_image() {
        let image = OwnedImage::random(Size::squared(16));
        assert_eq!(16 * 16, image.data.len());
        assert_eq!(16, image.get_width());
        assert_eq!(16, image.get_height());
    }
}
