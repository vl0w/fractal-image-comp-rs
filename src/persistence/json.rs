use serde::Serialize;
use crate::model::Compressed;

pub fn serialize(compressed: &Compressed) -> Vec<u8> {
    let contents = Contents::from(compressed.clone());
    // TODO: No unwrap!
    serde_json::to_string(&contents).unwrap().into_bytes()
}

#[derive(Serialize)]
struct Contents {
    w: u32,
    m: Vec<(Block, Block)>,
}

impl From<Compressed> for Contents {
    fn from(value: Compressed) -> Self {
        let transformations = value.0;
        let image_size = transformations[0].domain.image_size;
        Self {
            w: image_size,
            m: transformations.into_iter().map(|t| (Block::from(t.domain), Block::from(t.range))).collect(),
        }
    }
}


#[derive(Serialize)]
struct Block(u32, u32, u32);

impl From<crate::model::Block> for Block {
    fn from(value: crate::model::Block) -> Self {
        Self(value.block_size, value.origin.x, value.origin.y)
    }
}