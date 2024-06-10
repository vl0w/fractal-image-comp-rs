use crate::compress::quadtree::ErrorThreshold;
use clap::{ArgAction, Parser, Subcommand};
use std::path::{Path, PathBuf};
use indicatif::ProgressStyle;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use crate::image::Image;
use crate::preprocessing::SquaredGrayscaleImage;

mod compress;
mod decompress;
mod image;
mod model;
mod persistence;
mod preprocessing;
mod testutils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress {
        path: PathBuf,

        #[arg(short, long, action = ArgAction::SetTrue, help = "Reports progress" , default_value_t = false)]
        progress: bool,

        #[arg(
            short,
            long,
            required = false,
            help = "Sets the root mean squared error threshold for acceptable block mappings"
        )]
        rms_error_threshold: Option<f64>,
    },
    Decompress {
        path: PathBuf,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::FULL)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compress {
            path,
            progress,
            rms_error_threshold,
        } => {
            let image = SquaredGrayscaleImage::read_from(&path);
            info!("Image width: {}", image.get_width());
            info!("Image height: {}", image.get_height());

            let builder = compress::quadtree::Compressor::builder(image);
            let builder = if progress {
                let progress_bar = indicatif::ProgressBar::new(100)
                    .with_message("Mapping blocks")
                    .with_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})")
                        .unwrap()
                        .progress_chars("#>-"));

                builder.report_progress(move |progress| {
                    progress_bar.set_length(progress.total_area as u64);
                    if progress.finished() {
                        progress_bar.finish();
                    }
                    progress_bar.set_position(progress.area_covered as u64)
                })
            } else {
                builder
            };

            let builder = if let Some(rms_error_threshold) = rms_error_threshold {
                builder.with_error_threshold(ErrorThreshold::RmsAnyLowerThan(rms_error_threshold))
            } else {
                builder
            };

            let compressed = builder.build().compress();

            let size_of_file = compressed
                .persist_as_json(Path::new("transformations.json"))
                .expect("Could not save compression");

            info!(
                "Size of compression: {}",
                indicatif::HumanBytes(size_of_file)
            );
        }
        Commands::Decompress { .. } => {
            // let image = decompress::decompress(size, compressed);
            // image.save_image_as_png(Path::new("out.png"));
        }
    }
}
