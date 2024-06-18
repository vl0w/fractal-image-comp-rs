use derive_more::Display;
use std::sync::Arc;
pub use conversion::*;

use crate::image::iter::PixelIterator;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size};

#[derive(Display)]
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

impl<I: Image> SquaredBlock<SquaredBlock<I>> {
    pub fn flatten(self) -> SquaredBlock<I> {
        SquaredBlock {
            image: self.image.image.clone(),
            size: self.size,
            origin: self.origin + self.image.origin,
        }
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
    use std::sync::Arc;
    use itertools::Itertools;
    use crate::image::block::SquaredBlock;
    use crate::image::{Coords, Image};

    pub trait IntoSquaredBlocks<I>
    where
        I: Image + Send + Sync,
    {
        fn squared_blocks(self, size: u32) -> Vec<SquaredBlock<I>>;
    }

    impl<I> IntoSquaredBlocks<I> for I
    where
        I: Image,
    {
        fn squared_blocks(self, size: u32) -> Vec<SquaredBlock<I>> {
            create_squared_blocks(Arc::new(self), size)
        }
    }

    impl<I> IntoSquaredBlocks<I> for &Arc<I>
    where
        I: Image + Send + Sync,
    {
        fn squared_blocks(self, size: u32) -> Vec<SquaredBlock<I>> {
            create_squared_blocks(self.clone(), size)
        }
    }

    fn create_squared_blocks<I: Image>(image: Arc<I>, size: u32) -> Vec<SquaredBlock<I>> {
        assert_eq!(image.get_size().width % size, 0);
        assert_eq!(image.get_size().height % size, 0);
        assert_eq!(image.get_width(), image.get_height());

        let x_block = 0..image.get_width() / size;
        let y_block = 0..image.get_height() / size;

        x_block
            .cartesian_product(y_block)
            .map(move |(x, y)| SquaredBlock {
                size,
                origin: Coords {
                    x: size * y,
                    y: size * x,
                },
                image: Arc::clone(&image),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::image::fake::FakeImage;

    use super::*;

    #[test]
    #[should_panic]
    fn misaligned_block_size() {
        let _ = FakeImage::squared(16).squared_blocks(5);
    }

    #[test]
    #[should_panic]
    fn no_square_image() {
        let _ = FakeImage::new(Size::new(16, 32)).squared_blocks(4);
    }

    #[test]
    fn amount_of_blocks() {
        assert_eq!(FakeImage::squared(16).squared_blocks(16).len(), 1);
        assert_eq!(FakeImage::squared(16).squared_blocks(8).len(), 2 * 2);
        assert_eq!(FakeImage::squared(16).squared_blocks(4).len(), 4 * 4);
        assert_eq!(FakeImage::squared(16).squared_blocks(2).len(), 8 * 8);
        assert_eq!(FakeImage::squared(16).squared_blocks(1).len(), 16 * 16);
    }

    #[test]
    fn block_widths() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2);
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].get_width(), 2);
        assert_eq!(blocks[1].get_width(), 2);
        assert_eq!(blocks[2].get_width(), 2);
        assert_eq!(blocks[3].get_width(), 2);
    }

    #[test]
    fn block_heights() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2);
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].get_height(), 2);
        assert_eq!(blocks[1].get_height(), 2);
        assert_eq!(blocks[2].get_height(), 2);
        assert_eq!(blocks[3].get_height(), 2);
    }

    #[test]
    fn relative_pixel_values() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2);
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
        let blocks = image.squared_blocks(2);
        assert_eq!(blocks.len(), 4);
        blocks[0].pixel(2, 0);
    }

    #[test]
    #[should_panic]
    fn relative_pixel_values_overflow_y() {
        let image = FakeImage::squared(4);
        let blocks = image.squared_blocks(2);
        assert_eq!(blocks.len(), 4);
        blocks[0].pixel(0, 2);
    }

    #[test]
    fn twice() {
        // 0  1  2  3
        // 4  5  6  7
        // 8  9  10 11
        // 12 13 14 15

        let image = FakeImage::squared(4);
        let blocks: Vec<Arc<SquaredBlock<FakeImage>>> =
            image.squared_blocks(4).into_iter().map(Arc::new).collect();
        assert_eq!(blocks.len(), 1);
        let blocks: Vec<Arc<SquaredBlock<SquaredBlock<FakeImage>>>> = blocks[0]
            .squared_blocks(2)
            .into_iter()
            .map(Arc::new)
            .collect();
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

        let blocks = blocks[3].squared_blocks(1);
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
        let blocks: Vec<Arc<SquaredBlock<FakeImage>>> =
            image.squared_blocks(4).into_iter().map(Arc::new).collect();
        let mut inner_blocks = blocks[1].squared_blocks(2).into_iter();
        let _ = inner_blocks.next().unwrap();
        let _ = inner_blocks.next().unwrap();
        let third_block = inner_blocks.next().unwrap();

        assert_eq!(third_block.size, 2);
        assert_eq!(third_block.pixel(0, 0), 20);
        assert_eq!(third_block.pixel(1, 0), 21);
        assert_eq!(third_block.pixel(0, 1), 28);
        assert_eq!(third_block.pixel(1, 1), 29);

        let flattened = third_block.flatten();
        assert_eq!(flattened.size, 2);
        assert_eq!(flattened.pixel(0, 0), 20);
        assert_eq!(flattened.pixel(1, 0), 21);
        assert_eq!(flattened.pixel(0, 1), 28);
        assert_eq!(flattened.pixel(1, 1), 29);
    }
}
