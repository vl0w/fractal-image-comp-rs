use crate::image::Image;
use crate::image::Pixel;
use crate::image::Size;
use std::sync::Arc;

pub struct FakeImage {
    size: Size,
}

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
