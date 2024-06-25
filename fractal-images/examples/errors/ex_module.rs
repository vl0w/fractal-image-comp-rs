use std::fmt::Debug;

use cli_table::Table;

use fractal_image::{compress, decompress};
use fractal_image::image::{Image, IterablePixels, PowerOfTwo, Size, Square};
use fractal_image::preprocessing::SafeableImage;

#[derive(Table)]
pub struct Comparison {
    #[table(title = "Dimensions")]
    image_size: Size,
    #[table(title = "Filename")]
    file_name: String,
    #[table(title = "Compression size [Bytes]")]
    compressed_file_size_bytes: u64,
    #[table(title = "PNG size [Bytes]")]
    png_file_size_bytes: u64,
    #[table(title = "Ratio")]
    compression_ratio: f32,
}

pub fn compare_to_png_compression<I: Image + IterablePixels + Debug>(image: I) -> Comparison {
    let image_size = image.get_size();
    println!("Compressing image {}", image_size);
    let image = Square::new(image).expect("Image size needs to be square");
    let image = PowerOfTwo::new(image).expect("Image sizes need to be a power of two");

    let file_name = |prefix: &str| format!("{}_{}x{}",
                                           prefix,
                                           image_size.get_width(),
                                           image_size.get_height());

    let file_name_png = |prefix: &str| format!("{}.png", file_name(prefix));

    let original_file_name = file_name_png("orig");
    image.save_image_as_png(&original_file_name);
    let png_file_size = std::fs::metadata(&original_file_name).unwrap().len();

    let compressed = compress::quadtree::Compressor::new(image)
        .compress()
        .expect("Error while compressing image");

    let compressed_file_size = compressed.persist_as_binary_v1(file_name("cmp")).expect("Could not persist compressed image");
    let decompressed = decompress::decompress(compressed, decompress::Options::default());

    let out_file_name = file_name_png("out");
    decompressed.image.save_image_as_png(&out_file_name);

    Comparison {
        image_size,
        file_name: out_file_name,
        compressed_file_size_bytes: compressed_file_size,
        png_file_size_bytes: png_file_size,
        compression_ratio: compressed_file_size as f32 / png_file_size as f32,
    }
}