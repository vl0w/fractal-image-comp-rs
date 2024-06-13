#[cfg(test)]
use crate::image::Image;
#[cfg(test)]
use crate::image::Pixel;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
pub struct FakeImage {
    width: u32,
    height: u32,
}

#[cfg(test)]
impl Image for FakeImage {
    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.width);
        assert!(y < self.height);
        (y * self.width + x) as u8
    }
}

#[cfg(test)]
impl FakeImage {
    pub fn new(width: u32, height: u32) -> Arc<Self> {
        Arc::new(Self { width, height })
    }

    pub fn squared(size: u32) -> Arc<Self> {
        Self::new(size, size)
    }
}
