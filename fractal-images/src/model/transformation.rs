use crate::model::{Block, Rotation};

#[derive(Copy, Clone, Debug)]
pub struct Transformation {
    pub range: Block,
    pub domain: Block,
    pub rotation: Rotation,
    pub brightness: i16,
    pub saturation: f64,
}