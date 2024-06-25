use crate::coords;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size, Square};
use crate::image::iter::PixelIterator;

/// Generates a square
#[derive(Debug)]
pub struct GenSquare {
    image_size: Size,
    square_size: u32,
    center: Coords,
}

impl GenSquare {
    pub fn new(image_size: u32, square_size: u32) -> Square<Self> {
        let circle = Self {
            image_size: Size::squared(image_size),
            square_size,
            center: coords!(x=image_size/2, y = image_size/2),
        };
        Square::new(circle).unwrap()
    }
}

impl Image for GenSquare {
    fn get_size(&self) -> Size {
        self.image_size
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        let dx = self.center.x as i32 - x as i32;
        let dy = self.center.y as i32 - y as i32;
        let dx = dx.abs();
        let dy = dy.abs();

        let within_square = dx <= (self.square_size / 2) as i32 && dy <= (self.square_size / 2) as i32;

        if within_square {
            Pixel::MAX
        } else {
            0
        }
    }
}

impl IterablePixels for GenSquare {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        PixelIterator::new(self)
    }
}