use crate::model::Block;

#[derive(Copy, Clone, Debug)]
pub struct Transformation {
    pub range: Block,
    pub domain: Block,
    pub brightness: i16,
    pub saturation: f64,
}

#[derive(Debug, Clone)]
pub struct Compressed(pub Vec<Transformation>);

impl From<Vec<Transformation>> for Compressed {
    fn from(value: Vec<Transformation>) -> Self {
        Self(value)
    }
}