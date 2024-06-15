use crate::compress::quadtree::ErrorThreshold;
use clap::{ArgAction, Parser, Subcommand};
use indicatif::ProgressStyle;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use crate::image::Image;
use crate::model::Compressed;
use crate::preprocessing::{SafeableImage, SquaredGrayscaleImage};

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
        input_path: PathBuf,

        output_path: PathBuf,

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
        input_path: PathBuf,

        output_path: PathBuf,

        #[arg(short,long, default_value_t = 10)]
        iterations: u8,
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
            input_path,
            output_path,
            progress,
            rms_error_threshold,
        } => {
            let image = SquaredGrayscaleImage::read_from(&input_path);
            info!("Image width: {}", image.get_width());
            info!("Image height: {}", image.get_height());

            let compressor = compress::quadtree::Compressor::new(image);
            let compressor = if progress {
                let progress_bar = indicatif::ProgressBar::new(100)
                    .with_message("Mapping blocks")
                    .with_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len}")
                        .unwrap()
                        .progress_chars("#>-"));

                compressor.with_progress_reporter(move |progress| {
                    progress_bar.set_length(progress.total_area as u64);
                    if progress.finished() {
                        progress_bar.finish();
                    }
                    progress_bar.set_position(progress.area_covered as u64)
                })
            } else {
                compressor
            };

            let compressor = if let Some(rms_error_threshold) = rms_error_threshold {
                compressor
                    .with_error_threshold(ErrorThreshold::RmsAnyLowerThan(rms_error_threshold))
            } else {
                compressor
            };

            let compressed = compressor.compress();

            let size_of_file = compressed
                .persist_as_json(&output_path)
                .expect("Could not save compression");

            info!(
                "Size of compression: {}",
                indicatif::HumanBytes(size_of_file)
            );
        }
        Commands::Decompress {
            input_path,
            output_path,
            iterations
        } => {
            let compressed =
                Compressed::read_from_json(&input_path).expect("Could not read compressed file");
            // TODO: No -> Size needs to be part of Compressed!
            let image = decompress::decompress(compressed.0[0].domain.image_size, compressed, iterations);
            image.save_image_as_png(&output_path);
        }
    }
}
