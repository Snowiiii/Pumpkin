use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub client_translations: bool,
    // Reminder to update every new version, or until a better method is found
    pub translation_file_path: Option<PathBuf>,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            client_translations: true,
            translation_file_path: None,
        }
    }
}
