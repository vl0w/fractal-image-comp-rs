use std::fmt::Debug;
use derive_more::Display;
use std::sync::Arc;
pub use conversion::*;

use crate::image::iter::PixelIterator;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size};

#[derive(Display, Debug, Eq, PartialEq)]
#[display(fmt = "Block² {} {}", size, origin)]
pub struct SquaredBlock<I> {
    pub image: Arc<I>,

    pub size: u32,

    /// Represents the origin of the block, i.e. the `x` and `y` position in `image` where this block starts.
    pub origin: Coords,
}

impl<I> Clone for SquaredBlock<I> {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            size: self.size,
            origin: self.origin,
        }
    }
}

impl<I> SquaredBlock<I> {
    fn as_inner(&self) -> Arc<I> {
        self.image.clone()
    }
}

impl<I: Image> Image for SquaredBlock<I> {
    fn get_size(&self) -> Size {
        Size::squared(self.size)
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        assert!(x < self.size);
        assert!(y < self.size);
        self.image.pixel(self.origin.x + x, self.origin.y + y)
    }
}

impl<I: Image + Send + Sync> IterablePixels for SquaredBlock<I> {
    fn pixels_enumerated(&self) -> impl Iterator<Item=(Pixel, Coords)> {
        PixelIterator::new(self)
    }
}

/// Logic to turn something into [SquaredBlock]s.
mod conversion {
    use itertools::Itertools;
    use thiserror::Error;
    use crate::coords;
    use crate::image::block::SquaredBlock;
    use crate::image::{Coords, Image, Size, Square};
    use crate::model::Block;

    pub trait IntoSquaredBlocks<I> {
        fn squared_blocks(self, size: u32) -> Result<Vec<SquaredBlock<I>>, SquareSizeDoesNotDivideImageSize>;
    }

    #[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
    #[error(
        "The image with size {} can not be divided into blocks of size {}x{}. One of dimensions is not divisible by {}", .0, .1, .1, .1
    )]
    pub struct SquareSizeDoesNotDivideImageSize(Size, u32);

    type IntoSquaredBlocksResult<I> = Result<Vec<SquaredBlock<I>>, SquareSizeDoesNotDivideImageSize>;

    impl<I> IntoSquaredBlocks<I> for &Square<I>
    where
        I: Image,
    {
        fn squared_blocks(self, size: u32) -> IntoSquaredBlocksResult<I> {
            create_blocks(self.get_size(), size).map(|blocks| {
                blocks.map(|block| SquaredBlock {
                    image: self.as_inner(),
                    size,
                    origin: block.origin,
                }).collect::<Vec<_>>()
            })
        }
    }

    impl<I> IntoSquaredBlocks<I> for &SquaredBlock<I>
    where
        I: Image,
    {
        fn squared_blocks(self, size: u32) -> IntoSquaredBlocksResult<I> {
            create_blocks(self.get_size(), size).map(|blocks| {
                blocks.map(|block| SquaredBlock {
                    image: self.as_inner(),
                    size,
                    origin: block.origin + self.origin,
                }).collect::<Vec<_>>()
            })
        }
    }

    fn create_blocks(image_size: Size, size: u32) -> Result<impl Iterator<Item=Block>, SquareSizeDoesNotDivideImageSize> {
        assert!(image_size.is_squared());
        if image_size.get_width() % size != 0 || image_size.get_height() % size != 0 {
            return Err(SquareSizeDoesNotDivideImageSize(image_size, size));
        }

        let x_block = 0..image_size.get_width() / size;
        let y_block = 0..image_size.get_height() / size;

        Ok(x_block.cartesian_product(y_block).map(move |(x, y)| Block {
            block_size: size,
            origin: coords!(size * y, size * x),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::image::fake::FakeImage;

    use super::*;

    #[test]
    fn amount_of_blocks() {
        assert_eq!(FakeImage::squared(16).squared_blocks(16).unwrap().len(), 1);
        assert_eq!(FakeImage::squared(16).squared_blocks(8).unwrap().len(), 2 * 2);
        assert_eq!(FakeImage::squared(16).squared_blocks(4).unwrap().len(), 4 * 4);
        assert_eq!(FakeImage::squared(16).squared_blocks(2).unwrap().len(), 8 * 8);
        assert_eq!(FakeImage::squared(16).squared_blocks(1).unwrap().len(), 16 * 16);
    }

    #[test]
    fn block_widths() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2).unwrap();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].get_width(), 2);
        assert_eq!(blocks[1].get_width(), 2);
        assert_eq!(blocks[2].get_width(), 2);
        assert_eq!(blocks[3].get_width(), 2);
    }

    #[test]
    fn block_heights() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2).unwrap();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].get_height(), 2);
        assert_eq!(blocks[1].get_height(), 2);
        assert_eq!(blocks[2].get_height(), 2);
        assert_eq!(blocks[3].get_height(), 2);
    }

    #[test]
    fn relative_pixel_values() {
        let image = Arc::new(FakeImage::squared(4));
        let blocks = image.squared_blocks(2).unwrap();
        assert_eq!(blocks.len(), 4);

        assert_eq!(blocks[0].pixel(0, 0), image.pixel(0, 0));
        assert_eq!(blocks[0].pixel(1, 0), image.pixel(1, 0));
        assert_eq!(blocks[0].pixel(0, 1), image.pixel(0, 1));
        assert_eq!(blocks[0].pixel(1, 1), image.pixel(1, 1));

        assert_eq!(blocks[1].pixel(0, 0), image.pixel(2, 0));
        assert_eq!(blocks[1].pixel(1, 0), image.pixel(3, 0));
        assert_eq!(blocks[1].pixel(0, 1), image.pixel(2, 1));
        assert_eq!(blocks[1].pixel(1, 1), image.pixel(3, 1));

        assert_eq!(blocks[2].pixel(0, 0), image.pixel(0, 2));
        assert_eq!(blocks[2].pixel(1, 0), image.pixel(1, 2));
        assert_eq!(blocks[2].pixel(0, 1), image.pixel(0, 3));
        assert_eq!(blocks[2].pixel(1, 1), image.pixel(1, 3));

        assert_eq!(blocks[3].pixel(0, 0), image.pixel(2, 2));
        assert_eq!(blocks[3].pixel(1, 0), image.pixel(3, 2));
        assert_eq!(blocks[3].pixel(0, 1), image.pixel(2, 3));
        assert_eq!(blocks[3].pixel(1, 1), image.pixel(3, 3));
    }

    #[test]
    #[should_panic]
    fn relative_pixel_values_overflow_x() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2).unwrap();
        assert_eq!(blocks.len(), 4);
        blocks[0].pixel(2, 0);
    }

    #[test]
    #[should_panic]
    fn relative_pixel_values_overflow_y() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2).unwrap();
        assert_eq!(blocks.len(), 4);
        blocks[0].pixel(0, 2);
    }

    #[test]
    fn twice() {
        // 0  1  2  3
        // 4  5  6  7
        // 8  9  10 11
        // 12 13 14 15

        let image = Arc::new(FakeImage::squared(4));
        let blocks =
            image.squared_blocks(4).unwrap().into_iter().map(Arc::new).collect::<Vec<_>>();
        assert_eq!(blocks.len(), 1);
        let blocks = blocks[0].as_ref()
            .squared_blocks(2)
            .unwrap()
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].pixel(0, 0), 0);
        assert_eq!(blocks[0].pixel(1, 0), 1);
        assert_eq!(blocks[0].pixel(0, 1), 4);
        assert_eq!(blocks[0].pixel(1, 1), 5);
        assert_eq!(blocks[1].pixel(0, 0), 2);
        assert_eq!(blocks[1].pixel(1, 0), 3);
        assert_eq!(blocks[1].pixel(0, 1), 6);
        assert_eq!(blocks[1].pixel(1, 1), 7);
        assert_eq!(blocks[2].pixel(0, 0), 8);
        assert_eq!(blocks[2].pixel(1, 0), 9);
        assert_eq!(blocks[2].pixel(0, 1), 12);
        assert_eq!(blocks[2].pixel(1, 1), 13);
        assert_eq!(blocks[3].pixel(0, 0), 10);
        assert_eq!(blocks[3].pixel(1, 0), 11);
        assert_eq!(blocks[3].pixel(0, 1), 14);
        assert_eq!(blocks[3].pixel(1, 1), 15);

        let blocks = blocks[3].squared_blocks(1).unwrap();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].pixel(0, 0), 10);
        assert_eq!(blocks[1].pixel(0, 0), 11);
        assert_eq!(blocks[2].pixel(0, 0), 14);
        assert_eq!(blocks[3].pixel(0, 0), 15);
    }

    #[test]
    fn flatten() {
        //  0  1  2  3  4  5  6  7
        //  8  9 10 11 12 13 14 15
        // 16 17 18 19 20 21 22 23
        // 24 25 26 27 28 29 30 31
        // 32 33 34 35 36 37 38 39
        // 40 41 42 43 44 45 46 47
        // 48 49 50 51 52 53 54 55
        // 56 57 58 59 60 61 62 63

        let image = FakeImage::squared(8);
        let blocks =
            image.squared_blocks(4).unwrap().into_iter().map(Arc::new).collect::<Vec<_>>();
        let mut inner_blocks = blocks[1].squared_blocks(2).unwrap().into_iter();
        let _ = inner_blocks.next().unwrap();
        let _ = inner_blocks.next().unwrap();
        let third_block = inner_blocks.next().unwrap();

        assert_eq!(third_block.size, 2);
        assert_eq!(third_block.pixel(0, 0), 20);
        assert_eq!(third_block.pixel(1, 0), 21);
        assert_eq!(third_block.pixel(0, 1), 28);
        assert_eq!(third_block.pixel(1, 1), 29);
    }
}
