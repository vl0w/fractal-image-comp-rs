/// A representation for a gray scale pixel value
pub type Pixel = u8;

pub trait Image {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn pixel(&self, x: u32, y: u32) -> Pixel;
}

pub trait IterablePixels {
    fn pixels(&self) -> impl Iterator<Item=Pixel>;
}

pub struct OwnedImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl OwnedImage {
    fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        OwnedImage {
            width,
            height,
            data,
        }
    }
}

impl Image for OwnedImage {
    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.width);
        assert!(y < self.height);
        let idx = (y * self.width + x) as usize;
        self.data[idx]
    }
}

pub trait IntoOwnedImage<'a, T: Image> {
    fn into_owned(self) -> OwnedImage;
}

impl<'a, T> IntoOwnedImage<'a, T> for T where T: Image + 'a {
    fn into_owned(self) -> OwnedImage {
        OwnedImage {
            width: self.get_width(),
            height: self.get_height(),
            data: self.pixels().collect()
        }
    }
}

pub mod iter {
    use crate::image::{Image, IterablePixels, Pixel};

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

    pub struct PixelIterator<'a, T: Image + 'a> {
        image: &'a T,
        next: Next,
    }

    impl<'a, T: Image> PixelIterator<'a, T> {
        fn new(image: &'a T) -> Self {
            PixelIterator { image, next: Next::Xy(0, 0) }
        }
    }

    impl<'a, T: Image> Iterator for PixelIterator<'a, T> {
        type Item = Pixel;
        fn next(&mut self) -> Option<Self::Item> {
            match self.next {
                Next::Done => None,
                Next::Xy(x, y) => {
                    self.next = self.next.next_index(self.image.get_width(), self.image.get_height());
                    Some(self.image.pixel(x, y))
                }
            }
        }
    }

    impl<T> IterablePixels for T where T: Image {
        fn pixels(&self) -> impl Iterator<Item=Pixel> {
            PixelIterator::new(self)
        }
    }
}