use crate::model::{Block, Rotation};

#[derive(Copy, Clone, Debug)]
pub struct Transformation {
    pub range: Block,
    pub domain: Block,
    pub rotation: Rotation,
    pub brightness: i16,
    pub saturation: f64,
}

impl Eq for Transformation {}

impl PartialEq for Transformation {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range &&
            self.domain == other.domain &&
            self.rotation == other.rotation &&
            self.brightness == other.brightness &&
            (self.saturation - other.saturation).abs() < f64::EPSILON
    }
}