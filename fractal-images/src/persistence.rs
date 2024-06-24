#[cfg(feature = "persist-as-json")]
mod json;
#[cfg(feature = "persist-as-binary-v1")]
pub mod binary_v1;

use crate::model::Compressed;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::io;
use thiserror::Error;
use tracing::debug;

#[derive(Debug)]
enum Format {
    #[cfg(feature = "persist-as-json")]
    Json,
    #[cfg(feature = "persist-as-binary-v1")]
    QuadtreeFicV1,
}

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[cfg(feature = "persist-as-json")]
    #[error("Error while deserializing JSON: {0}")]
    JSONDeserializationError(#[from] json::DeserializationError),

    #[cfg(feature = "persist-as-json")]
    #[error("Error while serializing JSON: {0}")]
    JSONSerializationError(#[from] json::SerializationError),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[cfg(feature = "persist-as-binary-v1")]
    #[error("Error while serializing as QFIC (v1): {0}")]
    BinaryV1SerializationError(#[from] binary_v1::SerializationError),

    #[cfg(feature = "persist-as-binary-v1")]
    #[error("Error while deserializing as QFIC (v1): {0}")]
    BinaryV1DeserializationError(#[from] binary_v1::DeserializationError),
}

impl Compressed {
    #[cfg(feature = "persist-as-json")]
    pub fn persist_as_json<T: AsRef<Path>>(&self, path: T) -> Result<u64, PersistenceError> {
        self.persist_with(Format::Json, path.as_ref())
    }

    #[cfg(feature = "persist-as-binary-v1")]
    pub fn persist_as_binary_v1<T: AsRef<Path>>(&self, path: T) -> Result<u64, PersistenceError> {
        self.persist_with(Format::QuadtreeFicV1, path.as_ref())
    }

    fn persist_with(&self, format: Format, path: &Path) -> Result<u64, PersistenceError> {
        debug!("Persisting as {:?}", format);
        let serialized: Vec<u8> = match format {
            #[cfg(feature = "persist-as-json")]
            Format::Json => json::serialize(self)?,
            #[cfg(feature = "persist-as-binary-v1")]
            Format::QuadtreeFicV1 => binary_v1::serialize(self)?,
        };
        
        let mut file = File::create(path)?;
        file.write_all(serialized.as_slice())?;
        file.sync_all()?;

        // TODO: Illegal unwrap
        Ok(file.metadata().unwrap().len())
    }

    #[cfg(feature = "persist-as-json")]
    pub fn read_from_json(path: &Path) -> Result<Self, PersistenceError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let compressed = json::deserialize(reader)?;
        Ok(compressed)
    }

    #[cfg(feature = "persist-as-binary-v1")]
    pub fn read_from_binary_v1(path: &Path) -> Result<Self, PersistenceError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let compressed = binary_v1::deserialize(reader)?;
        Ok(compressed)
    }
}
