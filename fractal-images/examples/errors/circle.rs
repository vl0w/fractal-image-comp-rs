mod ex_module;

use cli_table::{print_stdout, WithTitle};
use fractal_image::image::gen::GenCircle;

fn main() {
    let compressions = vec![8, 16, 32, 64, 128, 256, 512].into_iter()
        .map(|image_size| {
            let radius = image_size as f64 / 2.0;
            GenCircle::new(image_size, radius)
        })
        .map(ex_module::compare_to_png_compression)
        .collect::<Vec<_>>();

    assert!(print_stdout(compressions.with_title()).is_ok());
}
