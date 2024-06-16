mod json;

use crate::model::Compressed;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::io;
use thiserror::Error;
use tracing::debug;

#[derive(Debug)]
enum Format {
    Json,
}

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Error while deserializing JSON: {0}")]
    JSONDeserializationError(#[from] json::DeserializationError),

    #[error("Error while serializing JSON: {0}")]
    JSONSerializationError(#[from] json::SerializationError),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),
}

impl Compressed {
    pub fn persist_as_json(&self, path: &Path) -> Result<u64, PersistenceError> {
        self.persist_with(Format::Json, path)
    }

    fn persist_with(&self, format: Format, path: &Path) -> Result<u64, PersistenceError> {
        debug!("Persisting as {:?}", format);
        let serialized: Vec<u8> = match format {
            Format::Json => json::serialize(self)?,
        };

        // Write the JSON string to a file
        let mut file = File::create(path)?;
        file.write_all(serialized.as_slice())?;
        file.sync_all()?;

        Ok(file.metadata().unwrap().len())
    }

    pub fn read_from_json(path: &Path) -> Result<Self, PersistenceError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let compressed = json::deserialize(reader)?;
        Ok(compressed)
    }
}
