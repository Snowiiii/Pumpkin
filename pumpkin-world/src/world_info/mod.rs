use thiserror::Error;

use crate::level::SaveFile;

pub mod anvil;

pub(crate) trait WorldInfoReader {
    fn read_world_info(&self, save_file: &SaveFile) -> Result<WorldInfo, WorldInfoError>;
}

#[derive(Debug, PartialEq)]
pub struct WorldInfo {
    pub seed: i64,
    // TODO: Implement all fields
}

#[derive(Error, Debug)]
pub enum WorldInfoError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Info not found!")]
    InfoNotFound,
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<std::io::Error> for WorldInfoError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::InfoNotFound,
            value => Self::IoError(value),
        }
    }
}
