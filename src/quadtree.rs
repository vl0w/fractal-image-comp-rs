use crate::image::{Image, IntoOwnedImage};
use crate::quadtree::blocks::IntoSquaredBlocks;
use crate::quadtree::scaled::IntoLazily2x2Scaled;
use crate::readwrite::AsDynamicImage;

pub fn compress<'a, I: Image + 'a>(image: I) -> impl Image {
    let range_blocks = image.squared_blocks(4);
    // let _ = range_blocks.map(|b| b.downscale_2x2());
    image.downscale_2x2().into_owned()
}


mod scaled {
    use crate::image::{Image, Pixel};

    pub struct Lazily2x2Scaled<'a, I: Image> {
        image: &'a I,
    }

    impl<'a, I: Image> Image for Lazily2x2Scaled<'a, I> {
        fn get_width(&self) -> u32 {
            self.image.get_width() / 2
        }

        fn get_height(&self) -> u32 {
            self.image.get_height() / 2
        }

        fn pixel(&self, x: u32, y: u32) -> Pixel {
            let sum = self.image.pixel(2 * x, 2 * y) +
                self.image.pixel(2 * x + 1, 2 * y) +
                self.image.pixel(2 * x, 2 * y + 1) +
                self.image.pixel(2 * x + 1, 2 * y + 1);
            (0.25 * sum as f64) as Pixel
        }
    }

    pub trait IntoLazily2x2Scaled<'a, I> where I: Image + 'a {
        fn downscale_2x2(&'a self) -> Lazily2x2Scaled<'a, I>;
    }

    impl<'a, I> IntoLazily2x2Scaled<'a, I> for I where I: Image + 'a {
        fn downscale_2x2(&'a self) -> Lazily2x2Scaled<'a, I> {
            Lazily2x2Scaled {
                image: self
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::image::Image;
        use crate::quadtree::scaled::IntoLazily2x2Scaled;
        use crate::testutils::FakeImage;

        #[test]
        fn downscaled_size() {
            let image = FakeImage::new(16, 16);
            let scaled = image.downscale_2x2();
            assert_eq!(scaled.get_width(), 8);
            assert_eq!(scaled.get_height(), 8);
        }

        #[test]
        fn groups_2x2_pixels_of_original_image() {
            // Original image
            // 0  1  2  3
            // 4  5  6  7
            // 8  9  10 11
            // 12 13 14 15

            let image = FakeImage::new(4, 4);
            let scaled = image.downscale_2x2();
            assert_eq!(scaled.pixel(0, 0), (1 + 4 + 5) / 4);
            assert_eq!(scaled.pixel(1, 0), (2 + 3 + 6 + 7) / 4);
            assert_eq!(scaled.pixel(0, 1), (8 + 9 + 12 + 13) / 4);
            assert_eq!(scaled.pixel(1, 1), (10 + 11 + 14 + 15) / 4);
        }

        #[test]
        #[should_panic]
        fn overflow_x() {
            let image = FakeImage::new(4, 4);
            let scaled = image.downscale_2x2();
            scaled.pixel(2, 0);
        }

        #[test]
        #[should_panic]
        fn overflow_y() {
            let image = FakeImage::new(4, 4);
            let scaled = image.downscale_2x2();
            scaled.pixel(0, 2);
        }
    }
}

mod blocks {
    use itertools::Itertools;

    use crate::image::{Image, Pixel};

    pub struct SquaredBlock<'a, I: Image> {
        image: &'a I,
        size: u32,
        rel_x: u32,
        rel_y: u32,
    }

    impl<'a, I: Image> SquaredBlock<'a, I> {
        fn new(image: &'a I, size: u32, rel_x: u32, rel_y: u32) -> Self {
            assert!(size <= image.get_width());
            assert!(size <= image.get_height());
            assert_eq!(image.get_width() % size, 0);
            assert_eq!(image.get_height() % size, 0);

            Self {
                image,
                size,
                rel_x,
                rel_y,
            }
        }
    }

    impl<'a, I: Image> Image for SquaredBlock<'a, I> {
        fn get_width(&self) -> u32 {
            self.size
        }

        fn get_height(&self) -> u32 {
            self.size
        }

        fn pixel(&self, x: u32, y: u32) -> Pixel {
            assert!(x < self.size);
            assert!(y < self.size);
            self.image.pixel(self.rel_x + x, self.rel_y + y)
        }
    }


    pub trait IntoSquaredBlocks<'a, I> where I: Image + 'a {
        fn squared_blocks(&'a self, size: u32) -> impl Iterator<Item=SquaredBlock<'a, I>>;
    }

    impl<'a, I> IntoSquaredBlocks<'a, I> for I where I: Image + 'a {
        fn squared_blocks(&'a self, size: u32) -> impl Iterator<Item=SquaredBlock<'a, I>> {
            assert_eq!(self.get_width() % size, 0);
            assert_eq!(self.get_height() % size, 0);
            assert_eq!(self.get_width(), self.get_height());

            let x_block = 0..self.get_width() / size;
            let y_block = 0..self.get_height() / size;

            x_block.cartesian_product(y_block).map(move |(x, y)| {
                SquaredBlock {
                    size,
                    rel_y: size * x,
                    rel_x: size * y,
                    image: self,
                }
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::testutils::FakeImage;

        use super::*;

        #[test]
        #[should_panic]
        fn misaligned_block_size() {
            let _ = FakeImage::new(16, 16).squared_blocks(5);
        }

        #[test]
        #[should_panic]
        fn no_square_image() {
            let _ = FakeImage::new(16, 32).squared_blocks(4);
        }

        #[test]
        fn amount_of_blocks() {
            assert_eq!(FakeImage::new(16, 16).squared_blocks(16).size_hint().0, 1);
            assert_eq!(FakeImage::new(16, 16).squared_blocks(8).size_hint().0, 2 * 2);
            assert_eq!(FakeImage::new(16, 16).squared_blocks(4).size_hint().0, 4 * 4);
            assert_eq!(FakeImage::new(16, 16).squared_blocks(2).size_hint().0, 8 * 8);
            assert_eq!(FakeImage::new(16, 16).squared_blocks(1).size_hint().0, 16 * 16);
        }

        #[test]
        fn block_widths() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            assert_eq!(blocks[0].get_width(), 2);
            assert_eq!(blocks[1].get_width(), 2);
            assert_eq!(blocks[2].get_width(), 2);
            assert_eq!(blocks[3].get_width(), 2);
        }

        #[test]
        fn block_heights() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            assert_eq!(blocks[0].get_height(), 2);
            assert_eq!(blocks[1].get_height(), 2);
            assert_eq!(blocks[2].get_height(), 2);
            assert_eq!(blocks[3].get_height(), 2);
        }

        #[test]
        fn relative_pixel_values() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
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
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            blocks[0].pixel(2, 0);
        }

        #[test]
        #[should_panic]
        fn relative_pixel_values_overflow_y() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            blocks[0].pixel(0, 2);
        }
    }
}


#[cfg(test)]
mod tests {}