use file_guard::{FileGuard, Lock};

use super::{LevelLocker, LockError};

use std::{fs::File, io::Write, sync::Arc};

pub struct AnvilLevelLocker {
    _lock: Option<FileGuard<Arc<File>>>,
}

const SESSION_LOCK_FILE_NAME: &str = "session.lock";

const SNOWMAN: &[u8] = "☃".as_bytes();

impl LevelLocker<Self> for AnvilLevelLocker {
    fn look(folder: &crate::level::LevelFolder) -> Result<Self, LockError> {
        let file_path = folder.root_folder.join(SESSION_LOCK_FILE_NAME);
        let mut file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(file_path)
            .unwrap();
        // im not joking, mojang writes a snowman into the lock file
        file.write_all(SNOWMAN)
            .map_err(|_| LockError::FailedWrite)?;
        let file_arc = Arc::new(file);
        let lock = file_guard::try_lock(file_arc, Lock::Exclusive, 0, 1)
            .map_err(|_| LockError::AlreadyLocked(SESSION_LOCK_FILE_NAME.to_string()))?;
        Ok(Self { _lock: Some(lock) })
    }
}
