use crate::compress::quadtree::stats::{Stats, StatsReporting};
use crate::compress::Mapping;
use crate::image::block::{IntoSquaredBlocks, SquaredBlock};
use crate::image::downscale::IntoDownscaled;
use crate::image::Image;
use crate::model::{Block, Compressed, Transformation};
use rayon::prelude::*;
use std::sync::Arc;
use log::warn;
use tracing::{debug, info, instrument};

pub struct Compressor<I> {
    image: Arc<I>,
    error_threshold: ErrorThreshold,
    progress_fn: Option<Arc<dyn Fn(stats::StatsReporting) + Send + Sync>>,
    stats: Arc<Stats>,
}

impl<I> Compressor<I>
where
    I: Image + Send,
{
    pub fn new(image: I) -> Self {
        Self {
            error_threshold: ErrorThreshold::default(),
            progress_fn: None,
            stats: Arc::new(Stats::new(image.get_height())),
            image: Arc::new(image),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn compress(self) -> Compressed {
        let image_height = self.image.get_height();
        let image_width = self.image.get_width();
        assert_eq!(
            image_height, image_width,
            "Only square sized images supported"
        );
        info!("Compressing {}x{} image", image_height, image_width);

        let domain_block_size: u32 = self.image.get_height();
        let range_block_size: u32 = (self.image.get_height() as f64 / 2.0) as u32;

        let domain_blocks = self.image.squared_blocks(domain_block_size);
        let range_blocks = self
            .image
            .squared_blocks(range_block_size)
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<Arc<_>>>();

        debug!(
            "Domain blocks: {} with size {}x{}",
            domain_blocks.len(),
            domain_block_size,
            domain_block_size
        );
        debug!(
            "Range blocks: {} with size {}x{}",
            range_blocks.len(),
            range_block_size,
            range_block_size
        );

        range_blocks
            .par_iter()
            .map(|rb| self.find_transformations_recursive(rb.clone()))
            .flatten()
            .collect::<Vec<Transformation>>()
            .into()
    }

    fn find_transformations_recursive(&self, rb: Arc<SquaredBlock<I>>) -> Vec<Transformation> {
        debug!("Finding transformation for range block {}", rb);
        match Transformation::find(self.image.clone(), &rb, self.error_threshold) {
            Some(transformation) => {
                debug!("For range block {}, found best matching domain block", rb);

                if let Some(progress_fn) = self.progress_fn.clone() {
                    self.stats.report_block_mapped(rb.get_height());
                    progress_fn(self.stats.report());
                }

                vec![transformation]
            }
            None => {
                debug!("For range block {}, found no matching domain block", rb);
                if rb.get_height() <= 1 {
                    warn!("Unable to map range block {}", rb);
                    vec![]
                } else {
                    rb.squared_blocks((rb.get_height() as f64 / 2.0) as u32)
                        .into_par_iter()
                        .map(|nrb| nrb.flatten())
                        .map(Arc::new)
                        .map(|nrb| self.find_transformations_recursive(nrb))
                        .flatten()
                        .collect()
                }
            }
        }
    }

    pub fn with_error_threshold(mut self, error_threshold: ErrorThreshold) -> Self {
        self.error_threshold = error_threshold;
        self
    }

    pub fn with_progress_reporter<F: Fn(StatsReporting) + Send + Sync + 'static>(
        mut self,
        progress_fn: F,
    ) -> Self {
        self.progress_fn = Some(Arc::new(progress_fn));
        self
    }
}

impl Transformation {
    fn find<I: Image + Send>(
        image: Arc<I>,
        range_block: &SquaredBlock<I>,
        error_threshold: ErrorThreshold,
    ) -> Option<Self> {
        let range_block_size = range_block.size;
        let domain_block_size = 2 * range_block_size;

        let domain_blocks = image.squared_blocks(domain_block_size);

        let mapping = domain_blocks
            .par_iter()
            .map(|d| d.downscale_2x2())
            .map(|db| {
                let mapping = Mapping::compute(&db, range_block);
                debug!("Mapping: {:?}", mapping);
                (db, mapping)
            })
            .find_any(|(_, mapping)| match error_threshold {
                ErrorThreshold::RmsAnyLowerThan(acceptable_error) => {
                    mapping.error < acceptable_error
                }
            });

        if let Some((db, mapping)) = mapping {
            debug!("Using mapping: {:?}", mapping);
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

#[derive(Copy, Clone, Debug)]
pub enum ErrorThreshold {
    RmsAnyLowerThan(f64),
}

impl Default for ErrorThreshold {
    fn default() -> Self {
        Self::RmsAnyLowerThan(5.0)
    }
}

mod stats {
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone, Copy, Debug)]
    pub struct StatsReporting {
        pub area_covered: u32,
        pub total_area: u32,
    }

    impl StatsReporting {
        pub fn finished(&self) -> bool {
            self.area_covered == self.total_area
        }
    }

    /// Records the area of the image that has already been mapped
    pub struct Stats {
        pub image_size_squared: u32,
        pub area_covered: AtomicU32,
    }

    impl Stats {
        pub fn new(image_size: u32) -> Self {
            Self {
                image_size_squared: image_size * image_size,
                area_covered: AtomicU32::new(0),
            }
        }

        pub fn report_block_mapped(&self, range_block_size: u32) {
            self.area_covered
                .fetch_add(range_block_size * range_block_size, Ordering::SeqCst);
        }

        pub fn report(&self) -> StatsReporting {
            StatsReporting {
                area_covered: self.area_covered.load(Ordering::SeqCst),
                total_area: self.image_size_squared,
            }
        }
    }
}
