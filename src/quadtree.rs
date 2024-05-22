use crate::image::Image;

struct Options {
    min_quadtree_depth: u8,
    max_quadtree_depth: u8,
}

impl Options {
    fn new(min_quadtree_depth: u8, max_quadtree_depth: u8) -> Self {
        assert!(min_quadtree_depth <= max_quadtree_depth);
        Self {
            min_quadtree_depth,
            max_quadtree_depth,
        }
    }
}

fn compress<I: Image>(image: I, options: Options) {}


mod blocks {
    use itertools::Itertools;
    use crate::image::{Image, Pixel};

    struct SquaredBlock<'a, I: Image> {
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


    trait IntoSquaredBlocks<'a, I> where I: Image + 'a {
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

            assert_eq!(blocks[0].pixel(0,0), image.pixel(0,0));
            assert_eq!(blocks[0].pixel(1,0), image.pixel(1,0));
            assert_eq!(blocks[0].pixel(0,1), image.pixel(0,1));
            assert_eq!(blocks[0].pixel(1,1), image.pixel(1,1));

            assert_eq!(blocks[1].pixel(0,0), image.pixel(2,0));
            assert_eq!(blocks[1].pixel(1,0), image.pixel(3,0));
            assert_eq!(blocks[1].pixel(0,1), image.pixel(2,1));
            assert_eq!(blocks[1].pixel(1,1), image.pixel(3,1));

            assert_eq!(blocks[2].pixel(0,0), image.pixel(0,2));
            assert_eq!(blocks[2].pixel(1,0), image.pixel(1,2));
            assert_eq!(blocks[2].pixel(0,1), image.pixel(0,3));
            assert_eq!(blocks[2].pixel(1,1), image.pixel(1,3));

            assert_eq!(blocks[3].pixel(0,0), image.pixel(2,2));
            assert_eq!(blocks[3].pixel(1,0), image.pixel(3,2));
            assert_eq!(blocks[3].pixel(0,1), image.pixel(2,3));
            assert_eq!(blocks[3].pixel(1,1), image.pixel(3,3));
        }

        #[test]
        #[should_panic]
        fn relative_pixel_values_overflow_x() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            blocks[0].pixel(2,0);
        }

        #[test]
        #[should_panic]
        fn relative_pixel_values_overflow_y() {
            let image = FakeImage::new(4, 4);
            let blocks = image.squared_blocks(2).collect::<Vec<_>>();
            assert_eq!(blocks.len(), 4);
            blocks[0].pixel(0,2);
        }

        struct FakeImage {
            width: u32,
            height: u32,
        }

        impl Image for FakeImage {
            fn get_width(&self) -> u32 {
                self.width
            }

            fn get_height(&self) -> u32 {
                self.height
            }

            fn pixel(&self, x: u32, y: u32) -> Pixel {
                assert!(x < self.width);
                assert!(y < self.height);
                (y * self.width + x) as u8
            }
        }

        impl FakeImage {
            fn new(width: u32, height: u32) -> Self {
                Self {
                    width,
                    height,
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {}