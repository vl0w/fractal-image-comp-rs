use std::path::Path;

use show_image::AsImageView;

use crate::image::Image;
use crate::readwrite::{AsDynamicImage, SafeableImage, SquaredGrayscaleImage};

mod quadtree;
mod image;
mod testutils;
mod readwrite;

fn main() {
    // let image = ::image::open(PathBuf::from("mrkrabs.png")).expect("Could not load image");
    // let view = image.as_image_view().expect("Could not initialize image view");
    // let window = create_window("Image", Default::default()).expect("Could not create window");
    // window.set_image("Image", view).expect("Could not set image");
    // Ok(())

    let image = SquaredGrayscaleImage::read_from(Path::new("mrkrabs.png"));
    println!("Image width: {}", image.get_width());
    println!("Image height: {}", image.get_height());
    let image = quadtree::compress(image);
    image.save_image_as_png(Path::new("out.png"));
    // let view = back.as_image_view().expect("Could not initialize image view");
    // let window = create_window("Image", Default::default()).expect("Could not create window");
    // window.set_image("Image", view).expect("Could not set image");
    // Ok(())
}
