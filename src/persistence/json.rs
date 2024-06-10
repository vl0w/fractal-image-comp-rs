use std::io::Read;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{coords, model};
use crate::image::Coords;
use crate::model::{Compressed, Transformation};

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("An error occurred while serializing: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn serialize(compressed: &Compressed) -> Result<Vec<u8>, SerializationError> {
    let contents = Contents::from(compressed.clone());
    let serialized = serde_json::to_string(&contents)?;
    Ok(serialized.into_bytes())
}
#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("An error occurred while deserializing: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub fn deserialize(reader: impl Read) -> Result<Compressed, DeserializationError> {
    let contents: Contents = serde_json::from_reader(reader)?;
    let transformations = contents
        .m
        .into_iter()
        .map(|(domain, range, brightness, saturation)| Transformation {
            range: model::Block {
                block_size: range.0,
                image_size: contents.w,
                origin: coords!(range.1, range.2),
            },
            domain: model::Block {
                block_size: domain.0,
                image_size: contents.w,
                origin: coords!(domain.1, domain.2),
            },
            brightness,
            saturation,
        })
        .collect();

    Ok(Compressed(transformations))
}

#[derive(Serialize, Deserialize)]
struct Contents {
    w: u32,
    m: Vec<(Block, Block, i16, f64)>,
}

impl From<Compressed> for Contents {
    fn from(value: Compressed) -> Self {
        let transformations = value.0;
        // TODO: No!
        let image_size = transformations[0].domain.image_size;
        Self {
            w: image_size,
            m: transformations
                .into_iter()
                .map(|t| {
                    (
                        Block::from(t.domain),
                        Block::from(t.range),
                        t.brightness,
                        t.saturation,
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Block(u32, u32, u32);

impl From<crate::model::Block> for Block {
    fn from(value: crate::model::Block) -> Self {
        Self(value.block_size, value.origin.x, value.origin.y)
    }
}
