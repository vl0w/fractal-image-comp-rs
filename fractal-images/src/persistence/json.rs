use std::io::Read;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::image::Coords;
use crate::{coords, model};

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("An error occurred while serializing: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn serialize(compressed: &model::Compressed) -> Result<Vec<u8>, SerializationError> {
    let contents = Contents::from(compressed.clone());
    let serialized = serde_json::to_string(&contents)?;
    Ok(serialized.into_bytes())
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("An error occurred while deserializing: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub fn deserialize(reader: impl Read) -> Result<model::Compressed, DeserializationError> {
    let contents: Contents = serde_json::from_reader(reader)?;
    let transformations = contents
        .mappings
        .into_iter()
        .map(|x| model::Transformation {
            range: model::Block {
                block_size: x.range.size,
                origin: coords!(x.range.x, x.range.y),
            },
            domain: model::Block {
                block_size: x.domain.size,
                origin: coords!(x.domain.x, x.domain.y),
            },
            rotation: model::Rotation::try_from(x.rotation.0)
                .unwrap_or(model::Rotation::By0),
            brightness: x.brightness,
            saturation: x.saturation,
        })
        .collect();

    Ok(model::Compressed {
        width: contents.width,
        height: contents.height,
        transformations,
    })
}

#[derive(Serialize, Deserialize)]
struct Contents {
    width: u32,
    height: u32,
    mappings: Vec<Mapping>,
}

impl From<model::Compressed> for Contents {
    fn from(compressed: model::Compressed) -> Self {
        Self {
            width: compressed.width,
            height: compressed.height,
            mappings: compressed
                .transformations
                .into_iter()
                .map(Mapping::from)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Mapping {
    domain: Block,
    range: Block,
    rotation: Rotation,
    brightness: i16,
    saturation: f64,
}

impl From<model::Transformation> for Mapping {
    fn from(value: model::Transformation) -> Self {
        Self {
            domain: Block::from(value.domain),
            range: Block::from(value.range),
            rotation: Rotation::from(value.rotation),
            brightness: value.brightness,
            saturation: value.saturation,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Block {
    size: u32,
    x: u32,
    y: u32,
}

impl From<model::Block> for Block {
    fn from(value: model::Block) -> Self {
        Self {
            size: value.block_size,
            x: value.origin.x,
            y: value.origin.y,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Rotation(u8);

impl From<model::Rotation> for Rotation {
    fn from(value: model::Rotation) -> Self {
        Self(value.try_into().unwrap_or(0))
    }
}