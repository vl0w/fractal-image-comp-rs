use crate::image::iter::PixelIterator;
use crate::image::{Coords, Image, IterablePixels, MutableImage, Pixel, Size};
use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct OwnedImage {
    size: Size,
    data: Vec<u8>,
}

impl OwnedImage {
    pub fn random(size: Size) -> Self {
        let mut data = Vec::with_capacity((size.area()) as usize);

        for _ in 0..(size.area()) {
            data.push(thread_rng().gen_range(0..256) as Pixel);
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

impl IterablePixels for OwnedImage {
    fn pixels_enumerated(&self) -> impl Iterator<Item = (Pixel, Coords)> {
        PixelIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_random_owned_image() {
        let image = OwnedImage::random(16);
        assert_eq!(16 * 16, image.data.len());
        assert_eq!(16, image.get_width());
        assert_eq!(16, image.get_height());
    }
}
