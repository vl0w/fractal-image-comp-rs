#[cfg(test)]
use crate::image::Image;
#[cfg(test)]
use crate::image::Pixel;
#[cfg(test)]
use crate::image::Size;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
pub struct FakeImage {
    size: Size,
}

#[cfg(test)]
impl Image for FakeImage {
    fn get_size(&self) -> Size {
        self.size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.get_width());
        assert!(y < self.get_height());
        (y * self.get_width() + x) as u8
    }
}

#[cfg(test)]
impl FakeImage {
    pub fn new(width: u32, height: u32) -> Arc<Self> {
        Arc::new(Self {
            size: Size::new(width, height),
        })
    }

    pub fn squared(size: u32) -> Arc<Self> {
        Self::new(size, size)
    }
}
