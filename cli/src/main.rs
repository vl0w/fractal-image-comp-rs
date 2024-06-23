use clap::{ArgAction, Parser, Subcommand};
use indicatif::ProgressStyle;
use std::ffi::OsStr;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use fractal_image::image::Image;
use fractal_image::model::Compressed;
use fractal_image::preprocessing::{SafeableImage, SquaredGrayscaleImage};
use fractal_image::{compress, decompress};

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
    /// Decompresses a compressed image as a PNG file.
    Decompress {
        /// The path (including a file name) of the compressed image.
        input_path: PathBuf,

        /// The path (including a file name) where the decompressed image should be saved.
        output_path: PathBuf,

        /// The amount of iterations to use for decompression.
        #[arg(short, long, default_value_t = 10)]
        iterations: u8,

        /// Stores the intermediate decompression results for each iteration.
        #[arg(short, long, default_value_t = false)]
        keep: bool,
    },
}

fn main() -> anyhow::Result<()> {
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
                compressor.with_error_threshold(
                    compress::quadtree::ErrorThreshold::RmsAnyLowerThan(rms_error_threshold),
                )
            } else {
                compressor
            };

            let compressed = compressor.compress()?;

            let size_of_file = compressed
                .persist_as_binary_v1(&output_path)
                .expect("Could not save compression");

            info!(
                "Size of compression: {}",
                indicatif::HumanBytes(size_of_file)
            );

            Ok(())
        }
        Commands::Decompress {
            input_path,
            output_path,
            iterations,
            keep,
        } => {
            let compressed =
                Compressed::read_from_binary_v1(&input_path).expect("Could not read compressed file");
            let decompressed = decompress::decompress(
                compressed,
                decompress::Options {
                    iterations,
                    keep_each_iteration: keep,
                },
            );

            if let Some(iterations) = &decompressed.iterations {
                let original_file_name = output_path
                    .file_stem()
                    .unwrap_or(OsStr::new("decompressed"))
                    .to_str()
                    .expect("Unable to process this file name");
                let extension = output_path
                    .extension()
                    .unwrap_or(OsStr::new("png"))
                    .to_str()
                    .expect("Unable to process this file extension");
                iterations
                    .iter()
                    .enumerate()
                    .map(|(index, image)| {
                        (
                            format!("{}.{}.{}", original_file_name, index, extension),
                            image,
                        )
                    })
                    .map(|(new_file_name, image)| {
                        (output_path.with_file_name(new_file_name), image)
                    })
                    .for_each(|(new_file_path, image)| image.save_image_as_png(&new_file_path))
            }

            decompressed.image.save_image_as_png(&output_path);
            
            Ok(())
        }
    }
}
