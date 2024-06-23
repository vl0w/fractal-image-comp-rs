use derive_more::Display;
use std::ops::{Add, Div, Mul};

mod block;
mod downscale;
mod owned;
mod rotate;
mod square;
mod fake;
mod power_of_two;

pub use block::*;
pub use downscale::*;
pub use owned::*;
pub use rotate::*;
pub use square::*;
pub use fake::*;
pub use power_of_two::*;


/// A representation for a gray scale pixel value
pub type Pixel = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display(fmt = "{}x{}", width, height)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn squared(size: u32) -> Self {
        Self::new(size, size)
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn get_width(&self) -> u32 { self.width }

    pub fn get_height(&self) -> u32 { self.height }

    /// Returns `true` iff `self.width == self.height`
    pub fn is_squared(&self) -> bool { self.width == self.height }

    /// Swaps width and height of a size, i.e.
    /// # Example
    /// ```rust
    /// use fractal_image::image::Size;
    ///
    /// assert_eq!(
    ///     Size::new(1,2).transpose(),
    ///     Size::new(2,1)
    /// );
    /// ```
    pub fn transpose(&self) -> Self {
        Size::new(self.get_height(), self.get_width())
    }
}

impl Div<u32> for Size {
    type Output = Size;

    fn div(self, rhs: u32) -> Self::Output {
        Self {
            width: self.width / rhs,
            height: self.width / rhs,
        }
    }
}

impl Mul<u32> for Size {
    type Output = Size;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::Output {
            width: self.width * rhs,
            height: self.width * rhs,
        }
    }
}

impl Mul<Size> for u32 {
    type Output = Size;

    fn mul(self, rhs: Size) -> Self::Output {
        Self::Output {
            width: rhs.width * self,
            height: rhs.width * self,
        }
    }
}

/// A convenience macro to create [Size]s.
#[macro_export]
macro_rules! size {
    (w = $width:expr, h = $height:expr) => {
        Size::new($width, $height)
    };
    (width = $width:expr, height = $height:expr) => {
        Size::new($width, $height)
    };
    (h = $height:expr, w = $width:expr) => {
        Size::new($width, $height)
    };
    (height = $height:expr, width = $width:expr) => {
        Size::new($width, $height)
    };
}

/// Represents the coordinates of a pixel
#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
#[display(fmt = "(x={}, y={})", x, y)]
pub struct Coords {
    pub x: u32,
    pub y: u32,
}

/// A macro to create [Coords] of the form `(x,y)`.
#[macro_export]
macro_rules! coords {
    (x=$x: expr, y=$y: expr) => {
        Coords { x: $x, y: $y }
    };
    (y=$y: expr, x=$x: expr) => {
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

pub trait Image: Send + Sync {
    fn get_size(&self) -> Size;

    fn get_height(&self) -> u32 { self.get_size().height }

    fn get_width(&self) -> u32 {
        self.get_size().width
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel;
}

pub trait IterablePixels {
    fn pixels(&self) -> impl Iterator<Item=Pixel> {
        self.pixels_enumerated().map(|(pixel, _)| pixel)
    }

    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)>;
}

pub trait MutableImage {
    fn set_pixel(&mut self, x: u32, y: u32, value: Pixel);
}

pub mod iter {
    use super::*;

    #[derive(Copy, Clone)]
    enum Next {
        Done,
        Xy(Coords),
    }

    impl Next {
        fn next_index(&self, size: Size) -> Self {
            match self {
                Next::Done => Next::Done,
                Next::Xy(Coords { x, y }) => {
                    let mut nx = x + 1;
                    let mut ny = *y;
                    if nx >= size.width {
                        nx = 0;
                        ny += 1;
                    }

                    if ny >= size.height {
                        Next::Done
                    } else {
                        Next::Xy(coords!(x=nx, y=ny))
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
                next: Next::Xy(coords!(x=0, y=0)),
            }
        }
    }

    impl<'a, T: Image> Iterator for PixelIterator<'a, T> {
        type Item = (Pixel, Coords);
        fn next(&mut self) -> Option<Self::Item> {
            match self.next {
                Next::Done => None,
                Next::Xy(Coords { x, y }) => {
                    self.next = self.next.next_index(self.image.get_size());
                    Some((self.image.pixel(x, y), coords!(x=x, y=y)))
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

    #[test]
    fn transpose_size() {
        assert_eq!(
            size!(w=1, h=2).transpose(),
            size!(w=2, h=1)
        )
    }
}
