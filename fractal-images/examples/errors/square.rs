mod ex_module;

use cli_table::{print_stdout, WithTitle};
use fractal_image::image::gen::GenSquare;

fn main() {
    let compressions = vec![8, 16, 32, 64, 128, 256, 512, 1024].into_iter()
        .map(|image_size| {
            GenSquare::new(image_size, (image_size as f64 * 0.5) as u32)
        })
        .map(ex_module::compare_to_png_compression)
        .collect::<Vec<_>>();

    assert!(print_stdout(compressions.with_title()).is_ok());
}
