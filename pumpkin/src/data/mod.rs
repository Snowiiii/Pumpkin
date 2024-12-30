use std::{env, fs, path::Path};

use serde::{Deserialize, Serialize};

const DATA_FOLDER: &str = "data/";

pub mod op_data;

pub mod banlist_serializer;
pub mod banned_ip_data;
pub mod banned_player_data;

pub trait LoadJSONConfiguration {
    #[must_use]
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let exe_dir = env::current_dir().unwrap();
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            log::debug!("creating new data root folder");
            fs::create_dir(&data_dir).expect("Failed to create data root folder");
        }
        let path = data_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {path:?}"));

            serde_json::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse data config at {path:?}. Reason: {err}. This is probably caused by a config update. Just delete the old data config and restart.",
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(&path, serde_json::to_string_pretty(&content).unwrap()) {
                log::error!(
                    "Couldn't write default data config to {path:?}. Reason: {err}. This is probably caused by a config update. Just delete the old data config and restart.", 
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
        let exe_dir = env::current_dir().unwrap();
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            log::debug!("creating new data root folder");
            fs::create_dir(&data_dir).expect("Failed to create data root folder");
        }
        let path = data_dir.join(Self::get_path());

        let content = match serde_json::to_string_pretty(self) {
            Ok(content) => content,
            Err(err) => {
                log::warn!(
                    "Couldn't serialize operator data config to {:?}. Reason: {}",
                    path,
                    err
                );
                return;
            }
        };

        if let Err(err) = std::fs::write(&path, content) {
            log::warn!(
                "Couldn't write operator config to {:?}. Reason: {}",
                path,
                err
            );
        }
    }
}
