use derive_more::Display;
use std::ops::Add;

pub mod block;
pub mod downscale;
pub mod owned;

/// A representation for a gray scale pixel value
pub type Pixel = u8;

/// Represents the coordinates of a pixel
#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display(fmt = "(x={}, y={})", x, y)]
pub struct Coords {
    pub x: u32,
    pub y: u32,
}

#[macro_export]
macro_rules! coords {
    ($x: expr, $y: expr) => {
        Coords { x: $x, y: $y }
    };
}

impl Add<Coords> for Coords {
    type Output = Coords;

    fn add(self, rhs: Coords) -> Self::Output {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub trait Image: Send+Sync {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn pixel(&self, x: u32, y: u32) -> Pixel;
}

pub trait IterablePixels {
    fn pixels(&self) -> impl Iterator<Item = Pixel> {
        self.pixels_enumerated().map(|(pixel, _)| pixel)
    }

    fn pixels_enumerated(&self) -> impl Iterator<Item = (Pixel, Coords)>;
}

pub trait MutableImage {
    fn set_pixel(&mut self, x: u32, y: u32, value: Pixel);
}

pub mod iter {
    use crate::image::{Coords, Image, Pixel};

    #[derive(Copy, Clone)]
    enum Next {
        Done,
        Xy(u32, u32),
    }

    impl Next {
        fn next_index(&self, width: u32, height: u32) -> Self {
            match self {
                Next::Done => Next::Done,
                Next::Xy(x, y) => {
                    let mut nx = x + 1;
                    let mut ny = *y;
                    if nx >= width {
                        nx = 0;
                        ny += 1;
                    }

                    if ny >= height {
                        Next::Done
                    } else {
                        Next::Xy(nx, ny)
                    }
                }
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct PixelIterator<'a, T: Image + 'a> {
        image: &'a T,
        next: Next,
    }

    impl<'a, T: Image> PixelIterator<'a, T> {
        pub fn new(image: &'a T) -> Self {
            PixelIterator {
                image,
                next: Next::Xy(0, 0),
            }
        }
    }

    impl<'a, T: Image> Iterator for PixelIterator<'a, T> {
        type Item = (Pixel, Coords);
        fn next(&mut self) -> Option<Self::Item> {
            match self.next {
                Next::Done => None,
                Next::Xy(x, y) => {
                    self.next = self
                        .next
                        .next_index(self.image.get_width(), self.image.get_height());
                    Some((self.image.pixel(x, y), coords!(x, y)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_coords() {
        assert_eq!(
            Coords { x: 3, y: 4 } + Coords { x: 5, y: 6 },
            Coords { x: 3 + 5, y: 4 + 6 }
        );
    }
}
