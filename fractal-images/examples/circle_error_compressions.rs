use cli_table::{Cell, Style, Table};

use fractal_image::compress;
use fractal_image::decompress;
use fractal_image::image::{Circle, PowerOfTwo};
use fractal_image::preprocessing::SafeableImage;

fn main() {
    let compressions = vec![8, 16, 32, 64, 128, 256, 512].into_iter()
        .map(compress_circle)
        .collect::<Vec<_>>();

    let table = compressions.into_iter().map(|c|
    vec![
        format!("{}x{}", c.image_size, c.image_size).cell(),
        c.file_name.cell(),
        format!("{}B", c.compressed_file_size_bytes).cell(),
        format!("{}B", c.png_file_size_bytes).cell(),
        c.compression_ratio.cell(),
    ]
    ).table().title(vec!["Dimensions", "Filename", "Compression size", "PNG size", "Ratio"]).bold(true);

    let table_display = table.display().unwrap();
    println!("{}", table_display);
}

struct CompressionResult {
    file_name: String,
    image_size: u32,
    compressed_file_size_bytes: u64,
    png_file_size_bytes: u64,
    compression_ratio: f32,
}

fn compress_circle(image_size: u32) -> CompressionResult {
    let circle_radius = image_size as f64 / 2.0;
    let circle = Circle::new(image_size, circle_radius);
    let circle = PowerOfTwo::new(circle).expect("Image sizes need to be a power of two");

    circle.save_image_as_png(format!("orig_{}x{}.png", image_size, image_size));
    let png_file_size = std::fs::metadata(format!("in_{}x{}.png", image_size, image_size)).unwrap().len();

    let compressed = compress::quadtree::Compressor::new(circle)
        .compress()
        .expect("Error while compressing image");

    let compressed_file_size = compressed.persist_as_binary_v1("compressed_image").expect("Could not persist compressed image");

    let decompressed = decompress::decompress(compressed, decompress::Options::default());

    let file_name = format!("out_{}x{}.png", image_size, image_size);
    decompressed.image.save_image_as_png(&file_name);
    CompressionResult {
        image_size,
        file_name,
        compressed_file_size_bytes: compressed_file_size,
        png_file_size_bytes: png_file_size,
        compression_ratio: compressed_file_size as f32 / png_file_size as f32,
    }
}