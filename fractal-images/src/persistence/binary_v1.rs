//! Binary compression for quadtree compressed images.
//!
//! The binary format uses the following pattern:
//! 
//! `<image width><image height>(<range block size><amount of blocks><block>)*`
//! 
//! where
//! 
//! `<block> = <range block origin><domain block origin><rotation><brightness><saturation>`
//!
//! ## Important
//! Relies on the fact that every domain block is twice the size of a range block. 
//! Returns a [SerializationError] if this is violated.

use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::{coords, model};
use crate::image::{Coords, Size};
use crate::model::{Rotation, RotationInvalidError};

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Persistence layer expects a quadtree compression.\
    The size of the domain block needs to be twice as the size of a range block, but it was not
    ({} != 2 * {})
    ", .domain_size, .range_size)]
    InvalidBlockSize { range_size: u32, domain_size: u32 },
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    InvalidRotation(#[from] RotationInvalidError),
}

pub fn serialize(compressed: &model::Compressed) -> Result<Vec<u8>, SerializationError> {
    let mut result: Vec<u8> = Vec::new();
    result.write_u32::<LittleEndian>(compressed.size.get_width())?;
    result.write_u32::<LittleEndian>(compressed.size.get_height())?;

    let rb_to_trans_map = generate_entries(compressed)?;

    for (rb_size, entry) in rb_to_trans_map {
        result.write_u32::<LittleEndian>(rb_size)?;
        entry.serialize(&mut result)?;
    }
    Ok(result)
}

fn generate_entries(compressed: &model::Compressed) -> Result<fxhash::FxHashMap<u32, Entry>, SerializationError> {
    let mut rb_to_trans_map = fxhash::FxHashMap::default();
    for t in &compressed.transformations {
        if t.domain.block_size != 2 * t.range.block_size {
            return Err(SerializationError::InvalidBlockSize { range_size: t.range.block_size, domain_size: t.domain.block_size });
        }

        let range_size = t.range.block_size;

        let rb_entry = rb_to_trans_map.entry(range_size).or_insert(Entry {
            entries: vec![],
        });

        rb_entry.entries.push(EntryChild {
            rb_origin: t.range.origin,
            db_origin: t.domain.origin,
            rotation: t.rotation.into(),
            brightness: t.brightness,
            saturation: t.saturation,
        })
    }

    Ok(rb_to_trans_map)
}

#[tracing::instrument(skip(reader))]
pub fn deserialize(mut reader: impl Read) -> Result<model::Compressed, DeserializationError> {
    let width = reader.read_u32::<LittleEndian>().unwrap();
    let height = reader.read_u32::<LittleEndian>().unwrap();

    let mut transformations = vec![];

    while let Ok(range_size) = reader.read_u32::<LittleEndian>() {
        let rb_entry = Entry::deserialize(&mut reader)?;

        for rb_child in rb_entry.entries {
            transformations.push(
                model::Transformation {
                    range: model::Block {
                        block_size: range_size,
                        origin: rb_child.rb_origin,
                    },
                    domain: model::Block {
                        block_size: 2 * range_size,
                        origin: rb_child.db_origin,
                    },
                    rotation: Rotation::try_from(rb_child.rotation)?,
                    brightness: rb_child.brightness,
                    saturation: rb_child.saturation,
                }
            );
        }
    }

    Ok(model::Compressed {
        size: Size::new(width, height),
        transformations,
    })
}

struct Entry {
    entries: Vec<EntryChild>,
}

impl Entry {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), SerializationError> {
        buf.write_u32::<LittleEndian>(self.entries.len() as u32)?;
        for entry in &self.entries {
            entry.serialize(buf)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializationError> {
        let entries_count = reader.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(entries_count as usize);
        for _ in 0..entries_count {
            let entry = EntryChild::deserialize(reader)?;
            entries.push(entry);
        }
        Ok(Self {
            entries,
        })
    }
}

struct EntryChild {
    rb_origin: Coords,
    db_origin: Coords,
    rotation: u8,
    brightness: i16,
    saturation: f64,
}

impl EntryChild {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), SerializationError> {
        buf.write_u32::<LittleEndian>(self.rb_origin.x)?;
        buf.write_u32::<LittleEndian>(self.rb_origin.y)?;
        buf.write_u32::<LittleEndian>(self.db_origin.x)?;
        buf.write_u32::<LittleEndian>(self.db_origin.y)?;
        buf.write_u8(self.rotation)?;
        buf.write_i16::<LittleEndian>(self.brightness)?;
        buf.write_f64::<LittleEndian>(self.saturation)?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializationError> {
        let rb_origin_x = reader.read_u32::<LittleEndian>()?;
        let rb_origin_y = reader.read_u32::<LittleEndian>()?;
        let db_origin_x = reader.read_u32::<LittleEndian>()?;
        let db_origin_y = reader.read_u32::<LittleEndian>()?;
        let rotation = reader.read_u8()?;
        let brightness = reader.read_i16::<LittleEndian>()?;
        let saturation = reader.read_f64::<LittleEndian>()?;

        Ok(Self {
            rb_origin: coords!(x=rb_origin_x, y=rb_origin_y),
            db_origin: coords!(x=db_origin_x, y=db_origin_y),
            rotation,
            brightness,
            saturation,
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use fluid::prelude::*;

    use crate::model::{Block, Compressed, Rotation, Transformation};
    use crate::size;

    use super::*;

    #[test]
    fn no_transformations() {
        let compressed = Compressed {
            size: size!(w=123, h=456),
            transformations: vec![],
        };

        let serialized = serialize(&compressed).unwrap();
        let cursor = Cursor::new(serialized);
        let deserialized = deserialize(cursor).unwrap();
        assert_eq!(deserialized.size, size!(w=123, h=456));
        assert!(deserialized.transformations.is_empty())
    }

    #[fact]
    fn one_transformation() {
        let transformation = create_transformation();
        let compressed = Compressed {
            size: size!(w=123, h=456),
            transformations: vec![transformation],
        };

        let serialized = serialize(&compressed).unwrap();
        let deserialized = deserialize(Cursor::new(serialized)).unwrap();
        deserialized.size.should().be_equal_to(size!(w=123, h= 456));
        deserialized.transformations.len().should().be_equal_to(1);
        deserialized.transformations[0].should().be_equal_to(transformation);
    }

    #[fact]
    fn multiple_transformations_should_be_compressable_and_decompressable() {
        let mut t_16_1 = create_transformation();
        t_16_1.range.block_size = 16;
        t_16_1.domain.block_size = 32;
        let mut t_16_2 = create_transformation();
        t_16_2.range.block_size = 16;
        t_16_2.domain.block_size = 32;
        let mut t_32_1 = create_transformation();
        t_32_1.range.block_size = 32;
        t_32_1.domain.block_size = 64;
        let compressed = Compressed {
            size: size!(w=123, h=456),
            transformations: vec![t_16_1, t_16_2, t_32_1],
        };

        let serialized = serialize(&compressed).unwrap();
        let deserialized = deserialize(Cursor::new(serialized)).unwrap();
        deserialized.size.should().be_equal_to(size!(w=123, h= 456));
        deserialized.transformations.len().should().be_equal_to(3);
        deserialized.transformations[0].should().be_equal_to(t_16_1);
        deserialized.transformations[1].should().be_equal_to(t_16_2);
        deserialized.transformations[2].should().be_equal_to(t_32_1);
    }

    #[fact]
    fn invalid_domain_block_size_returns_error() {
        let mut transformation = create_transformation();
        transformation.domain.block_size *= 2;
        let compressed = Compressed {
            size: size!(w=123, h=456),
            transformations: vec![transformation],
        };

        serialize(&compressed).should().be_an_error()
            .because("the domain block size is not twice the range block size");
    }

    fn create_transformation() -> Transformation {
        Transformation {
            range: Block {
                block_size: 16,
                origin: coords!(x=rand::random(), y=rand::random()),
            },
            domain: Block {
                block_size: 32,
                origin: coords!(x=rand::random(), y=rand::random()),
            },
            rotation: Rotation::By0,
            brightness: rand::random(),
            saturation: rand::random(),
        }
    }
}