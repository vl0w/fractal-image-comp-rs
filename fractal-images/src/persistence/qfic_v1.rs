use std::io::Read;

use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;
use thiserror::Error;

use crate::{coords, model};
use crate::image::{Coords, Size};
use crate::model::Rotation;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error)
}

pub fn serialize(compressed: &model::Compressed) -> Result<Vec<u8>, SerializationError> {
    let mut result: Vec<u8> = Vec::new();
    result.write_u32::<LittleEndian>(compressed.size.get_width())?;
    result.write_u32::<LittleEndian>(compressed.size.get_height())?;

    let rb_to_trans_map = generate_map(compressed)?;

    for (rb_size, entry) in rb_to_trans_map {
        result.write_u32::<LittleEndian>(rb_size)?;
        entry.serialize(&mut result)?;
    }
    Ok(result)
}

fn generate_map(compressed: &model::Compressed) -> Result<fxhash::FxHashMap<u32, RbEntry>, SerializationError> {
    let mut rb_to_trans_map = fxhash::FxHashMap::default();
    for t in &compressed.transformations {
        // TODO: Check that domain block size is 2*range_block_size
        let range_size = t.range.block_size;

        let rb_entry = rb_to_trans_map.entry(range_size).or_insert(RbEntry {
            amount: 0,
            entries: vec![],
        });

        rb_entry.amount += 1;
        rb_entry.entries.push(RbEntryChild {
            rb_origin: t.range.origin,
            db_origin: t.domain.origin,
            rotation: t.rotation.into(),
            brightness: t.brightness,
            saturation: t.saturation,
        })
    }

    Ok(rb_to_trans_map)
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error)
}

pub fn deserialize(mut reader: impl Read) -> Result<model::Compressed, DeserializationError> {
    let width = reader.read_u32::<LittleEndian>().unwrap();
    let height = reader.read_u32::<LittleEndian>().unwrap();

    let mut transformations = vec![];


    while let Ok(range_size) = reader.read_u32::<LittleEndian>() {
        // let transformations_count = reader.read_u32::<LittleEndian>()?;
        // for _ in 0..transformations_count {
        let rb_entry = RbEntry::deserialize(&mut reader)?;

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
                    // TODO: no unwrap!
                    rotation: Rotation::try_from(rb_child.rotation).unwrap(),
                    brightness: rb_child.brightness,
                    saturation: rb_child.saturation,
                }
            );
            // }
        }
    }

    Ok(model::Compressed {
        size: Size::new(width, height),
        transformations,
    })
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

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

    #[test]
    fn one_transformation() {
        let transformation = Transformation {
            range: Block {
                block_size: 16,
                origin: coords!(x=1, y=2),
            },
            domain: Block {
                block_size: 32,
                origin: coords!(x=3, y=4),
            },
            rotation: Rotation::By0,
            brightness: 5,
            saturation: 6.7,
        };
        let compressed = Compressed {
            size: size!(w=123, h=456),
            transformations: vec![transformation],
        };

        let serialized = serialize(&compressed).unwrap();
        let cursor = Cursor::new(serialized);
        let deserialized = deserialize(cursor).unwrap();
        assert_eq!(deserialized.size, size!(w=123, h=456));
        assert_eq!(deserialized.transformations.len(), 1);
        assert_eq!(deserialized.transformations[0], transformation);
    }

    #[test]
    #[ignore]
    fn multiple_transformations() {
        todo!()
    }

    #[test]
    #[ignore]
    fn compress_invalid_domain_block_size() {
        todo!()
    }
}

struct RbEntry {
    // TODO: Remove :)
    amount: u32,
    entries: Vec<RbEntryChild>,
}

impl RbEntry {
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
            let entry = RbEntryChild::deserialize(reader)?;
            entries.push(entry);
        }
        Ok(Self {
            amount: entries_count,
            entries,
        })
    }
}
//
// impl TryFrom<String> for RbEntry {
//     type Error = anyhow::Error;
//
//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         let splitted: Vec<&str> = value.split(',').collect();
//         if splitted.len() < 1 {
//             bail!("???");
//         }
//
//         let amount = splitted[0].parse::<u32>().expect("nAn");
//
//         if splitted.len() != 1 + 7 * amount as usize {
//             bail!("nope")
//         }
//
//         let splitted = splitted.into_iter().dropping(1).collect::<Vec<&str>>();
//
//         if splitted.len() != 7 * amount as usize {
//             bail!("nope")
//         }
//
//         let entries = splitted
//             .chunks_exact(7)
//             .map(RbEntryChild::try_from)
//             .collect::<Result<Vec<RbEntryChild>, _>>()?;
//
//         Ok(Self {
//             amount,
//             entries
//         })
//     }
// }

struct RbEntryChild {
    rb_origin: Coords,
    db_origin: Coords,
    rotation: u8,
    brightness: i16,
    saturation: f64,
}

impl RbEntryChild {
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

impl TryFrom<&[&str]> for RbEntryChild {
    type Error = anyhow::Error;

    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        if value.len() != 7 {
            bail!("Invalid range block entry (size expected to be ???)")
        }

        let x = value[0].parse::<u32>().expect("nAn");
        let y = value[1].parse::<u32>().expect("nAn");
        let rb_coords = coords!(x=x, y=y);

        let x = value[2].parse::<u32>().expect("nAn");
        let y = value[3].parse::<u32>().expect("nAn");
        let db_coords = coords!(x=x, y=y);

        let rotation = value[4].parse::<u8>().expect("nAn");
        let brightness = value[5].parse::<i16>().expect("nAn");
        let saturation = value[6].parse::<f64>().expect("nAn");


        Ok(Self {
            rb_origin: rb_coords,
            db_origin: db_coords,
            rotation,
            brightness,
            saturation,
        })
    }
}


impl TryFrom<String> for RbEntryChild {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let splitted: Vec<&str> = value.split(',').collect();

        if splitted.len() != 7 {
            bail!("Invalid range block entry (size expected to be ???)")
        }

        let x = splitted[0].parse::<u32>().expect("nAn");
        let y = splitted[1].parse::<u32>().expect("nAn");
        let rb_coords = coords!(x=x, y=y);

        let x = splitted[2].parse::<u32>().expect("nAn");
        let y = splitted[3].parse::<u32>().expect("nAn");
        let db_coords = coords!(x=x, y=y);

        let rotation = splitted[4].parse::<u8>().expect("nAn");
        let brightness = splitted[5].parse::<i16>().expect("nAn");
        let saturation = splitted[6].parse::<f64>().expect("nAn");


        Ok(Self {
            rb_origin: rb_coords,
            db_origin: db_coords,
            rotation,
            brightness,
            saturation,
        })
    }
}