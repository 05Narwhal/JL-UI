use std::{
    error::Error, fs, path::PathBuf,
    io::{Error as IoError, ErrorKind}
};
use serde::{Serialize, de::DeserializeOwned};

/// Reads and writes JSON files, with error handling and logging.
pub fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T, Box<dyn Error>> {
    if let Ok(json_str) = fs::read_to_string(path) {
        match serde_json::from_str::<T>(&json_str) {
            Ok(data) => Ok(data),
            Err(e) => {
                crate::ui_error!("Failed to parse JSON from {}: {}", path.display(), e);
                Err(Box::new(e))
            }
        }
    } else {
        crate::ui_error!("Failed to read file: {}", path.display());
        Err(Box::new(IoError::new(ErrorKind::NotFound, "File not found")))
    }
}

/// Writes JSON data to a file, returning true on success or false on failure.
pub fn write_json<T: Serialize>(path: &PathBuf, data: &T) -> bool {
    match serde_json::to_string_pretty(data) {
        Ok(json_str) => {
            if let Err(e) = fs::write(path, json_str) {
                crate::ui_error!("Failed to write JSON to {}: {}", path.display(), e);
                false
            } else {
                true
            }
        }
        Err(e) => {
            crate::ui_error!("Failed to serialize data to JSON: {}", e);
            false
        }
    }
}