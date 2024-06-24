use fractal_image::compress;
use fractal_image::decompress;
use fractal_image::image::{Circle, PowerOfTwo};
use fractal_image::preprocessing::SafeableImage;

fn main() {
    let image_size = 512;
    let circle_radius = image_size as f64 / 2.0;
    let circle = Circle::new(image_size, circle_radius);
    let circle = PowerOfTwo::new(circle).expect("Image sizes need to be a power of two");

    let compressed = compress::quadtree::Compressor::new(circle)
        .compress()
        .expect("Error while compressing image");

    let decompressed = decompress::decompress(compressed, decompress::Options::default());

    decompressed.image.save_image_as_png("out.png");
}