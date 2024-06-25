use fractal_image::{compress, decompress, metrics};
use fractal_image::compress::quadtree::ErrorThreshold;
use fractal_image::image::{OwnedImage, PowerOfTwo, Size, Square};

enum TestImage {
    RandomNoise256x256
}

impl TestImage {
    fn generate(self) -> OwnedImage {
        match self {
            TestImage::RandomNoise256x256 =>
                OwnedImage::random(Size::squared(256))
        }
    }
}

#[test]
fn error_for_random_noise() {
    test_error(TestImage::RandomNoise256x256.generate(),
               ErrorThreshold::AnyBlockBelowRms(100.0),
               5454.0,
               10.76);
}

fn test_error(image: OwnedImage,
              error_threshold: ErrorThreshold,
              expected_mse: f64,
              expected_psnr: f64) {
    let image = Square::new(image).unwrap();
    let image = PowerOfTwo::new(image).unwrap();

    let compressor = compress::quadtree::Compressor::new(image.clone())
        .with_error_threshold(error_threshold);
    let compressed = compressor.compress().unwrap();

    let decompressed = decompress::decompress(compressed, decompress::Options::default());
    let decompressed_image = decompressed.image;

    let mse = metrics::mse(&image, &decompressed_image).unwrap();
    let psnr = metrics::psnr(&image, &decompressed_image).unwrap();
    assert_within_bounds(mse, expected_mse, "mse");
    assert_within_bounds(psnr, expected_psnr, "psnr");
}

fn assert_within_bounds(actual: f64, expected: f64, name: &str) {
    let lower_bound = 0.99 * expected;
    let upper_bound = 1.01 * expected;

    assert!(lower_bound <= actual, "Expected {} <= {} <= {}, was {}", lower_bound, name, upper_bound, actual);
    assert!(actual <= upper_bound, "Expected {} <= {} <= {}, was {}", lower_bound, name, upper_bound, actual);
}
