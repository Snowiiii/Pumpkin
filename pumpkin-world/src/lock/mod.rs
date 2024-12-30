use thiserror::Error;

use crate::level::LevelFolder;

pub mod anvil;

// Gets unlocked when dropped
pub trait LevelLocker<T>: Send + Sync {
    fn look(folder: &LevelFolder) -> Result<T, LockError>;
}

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Oh no, Level is already locked by {0}")]
    AlreadyLocked(String),
    #[error("Failed to write into lock file")]
    FailedWrite,
}
