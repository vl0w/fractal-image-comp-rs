use crate::coords;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size, Square};
use crate::image::iter::PixelIterator;

/// Generates a circle
#[derive(Debug)]
pub struct GenCircle {
    image_size: Size,
    radius: f64,
    center: Coords,
}

impl GenCircle {
    pub fn new(image_size: u32, radius: f64) -> Square<Self> {
        let circle = Self {
            image_size: Size::squared(image_size),
            radius,
            center: coords!(x=image_size/2, y = image_size/2),
        };
        Square::new(circle).unwrap()
    }
}

impl Image for GenCircle {
    fn get_size(&self) -> Size {
        self.image_size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let dx = self.center.x as i32 - x as i32;
        let dy = self.center.y as i32 - y as i32;
        let dx = dx as f64;
        let dy = dy as f64;

        let within_circle = (dx * dx + dy * dy).sqrt() <= self.radius;

        if within_circle {
            Pixel::MAX
        } else {
            0
        }
    }
}

impl IterablePixels for GenCircle {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        PixelIterator::new(self)
    }
}