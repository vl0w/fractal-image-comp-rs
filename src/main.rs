use crate::compress::quadtree::ErrorThreshold;
use std::path::Path;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use crate::image::Image;
use crate::preprocessing::{SafeableImage, SquaredGrayscaleImage};

mod compress;
mod decompress;
mod image;
mod model;
mod persistence;
mod preprocessing;
mod testutils;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::FULL)
        .init();

    let image = SquaredGrayscaleImage::read_from(Path::new("mrkrabs_1200x1200.png"));
    info!("Image width: {}", image.get_width());
    info!("Image height: {}", image.get_height());
    let size = image.get_width();
    let compressed = compress::quadtree::Compressor::builder(image)
        .report_progress(|progress| println!("{}", progress))
        .with_error_threshold(ErrorThreshold::RmsAnyLowerThan(50f64))
        .build()
        .compress();
    let size_of_file = compressed
        .persist_as_json(Path::new("transformations.json"))
        .expect("Could not save compression");
    info!("Size of compression: {}kB", size_of_file as f64 / 1024.0);
    let image = decompress::decompress(size, compressed);
    image.save_image_as_png(Path::new("out.png"));
}
