/// A representation for a gray scale pixel value
pub type Pixel = u8;

pub trait Image {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn pixel(&self, x: u32, y: u32) -> Pixel;
}