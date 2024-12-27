use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub mod op_data;

pub trait LoadJSONConfiguration {
    #[must_use]
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let path = Self::get_path();

        let config = if path.exists() {
            let file_content = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {path:?}"));

            serde_json::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {path:?}. Reason: {err}. This is probably caused by a config update. Just delete the old config and restart.",
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(path, serde_json::to_string_pretty(&content).unwrap()) {
                eprintln!(
                    "Couldn't write default config to {path:?}. Reason: {err}. This is probably caused by a config update. Just delete the old config and restart.", 
                );
            }

            content
        };

        config.validate();
        config
    }

    fn get_path() -> &'static Path;

    fn validate(&self);
}

pub trait SaveJSONConfiguration: LoadJSONConfiguration {
    // suppress clippy warning

    fn save(&self)
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let path = <Self as LoadJSONConfiguration>::get_path();

        let content = match serde_json::to_string_pretty(self) {
            Ok(content) => content,
            Err(err) => {
                log::warn!(
                    "Couldn't serialize operator config to {:?}. Reason: {}",
                    path,
                    err
                );
                return;
            }
        };

        if let Err(err) = std::fs::write(path, content) {
            log::warn!(
                "Couldn't write operator config to {:?}. Reason: {}",
                path,
                err
            );
        }
    }
}
