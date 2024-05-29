mod json;

use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use tracing::debug;
use crate::model::Compressed;

#[derive(Debug)]
enum Format {
    Json
}

impl Compressed {
    pub fn persist_as_json(&self, path: &Path) -> io::Result<u64> {
        self.persist_with(Format::Json, path)
    }

    fn persist_with(&self, format: Format, path: &Path) -> io::Result<u64> {
        debug!("Persisting as {:?}", format);
        let serialized: Vec<u8> = match format {
            Format::Json => json::serialize(self)
        };

        // Write the JSON string to a file
        let mut file = File::create(path)?;
        file.write_all(serialized.as_slice())?;
        file.sync_all()?;
        
        Ok(file.metadata().unwrap().len())
    }
}