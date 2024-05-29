use std::collections::VecDeque;
use std::rc::Rc;
use tracing::{debug, info, instrument};
use crate::compress::Mapping;
use crate::image::Image;
use crate::image::block::{IntoSquaredBlocks, SquaredBlock};
use crate::model::{Block, Compressed, Transformation};
use crate::image::downscale::IntoDownscaled;

#[instrument(level = "debug", skip(image))]
pub fn compress<I: Image>(image: I) -> Compressed {
    let image_height = image.get_height();
    let image_width = image.get_width();
    assert_eq!(image_height, image_width, "Only square sized images supported");
    info!("Compressing {}x{} image", image_height, image_width);
    let image = Rc::new(image);

    let mut transformations: Vec<Transformation> = Vec::new();

    let domain_block_size: u32 = image.get_height();
    let range_block_size: u32 = (image.get_height() as f64 / 2.0) as u32;

    let domain_blocks = image.squared_blocks(domain_block_size);
    let range_blocks = image.squared_blocks(range_block_size).into_iter().map(Rc::new).collect::<Vec<Rc<_>>>();

    debug!("Domain blocks: {} with size {}x{}", domain_blocks.len(), domain_block_size, domain_block_size);
    debug!("Range blocks: {} with size {}x{}", range_blocks.len(), range_block_size, range_block_size);

    let mut queue = VecDeque::from(range_blocks);

    while let Some(rb) = queue.pop_front() {
        debug!("Finding transformation for range block {} (remaining: {})", rb, queue.len());
        match Transformation::find(image.clone(), &rb) {
            Some(transformation) => {
                debug!("For range block {}, found best matching domain block", rb);
                transformations.push(transformation)
            }
            None => {
                debug!("For range block {}, found no matching domain block", rb);
                if rb.get_height() <= 1 {
                    panic!("Nope"); // TODO
                } else {
                    let new_range_blocks = rb.squared_blocks((rb.get_height() as f64 / 2.0) as u32);
                    let new_range_blocks = new_range_blocks.into_iter().map(|nrb| nrb.flatten());
                    new_range_blocks.into_iter().for_each(|nrb| queue.push_back(Rc::new(nrb)))
                }
            }
        }
    }

    transformations.into()
}

impl Transformation {
    fn find<I: Image>(image: Rc<I>, range_block: &SquaredBlock<I>) -> Option<Self> {
        let range_block_size = range_block.size;
        let domain_block_size = 2 * range_block_size;

        let domain_blocks = image.squared_blocks(domain_block_size);

        let mapping = domain_blocks.iter().map(|d| d.downscale_2x2()).map(|db| {
            let mapping = Mapping::compute(&db, range_block);
            debug!("Mapping: {:?}",mapping);
            (db, mapping)
        })
            .filter(|(_, mapping)| mapping.error < 50_f64)
            .take(1)
            .next();

        if let Some((db, mapping)) = mapping {
            debug!("Using mapping: {:?}",mapping);
            return Some(Self {
                range: Block {
                    block_size: range_block.size,
                    image_size: image.get_height(),
                    origin: range_block.origin,
                },
                domain: Block {
                    block_size: db.inner().size,
                    image_size: image.get_height(),
                    origin: db.inner().origin,
                },
                brightness: mapping.brightness,
                saturation: mapping.saturation,
            });
        }

        None
    }
}