use crate::model::Block;

#[derive(Copy, Clone, Debug)]
pub struct Transformation {
    pub range: Block,
    pub domain: Block,
    pub brightness: i16,
    pub saturation: f64,
}