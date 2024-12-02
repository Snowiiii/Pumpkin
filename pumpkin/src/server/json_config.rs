use std::{fs, path::Path, sync::LazyLock};

use log::warn;
use pumpkin_config::op;
use serde::{Deserialize, Serialize};

pub static OPERATOR_CONFIG: LazyLock<tokio::sync::RwLock<OperatorConfig>> =
    LazyLock::new(|| tokio::sync::RwLock::new(OperatorConfig::load()));


#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct OperatorConfig {
    pub ops: Vec<op::Op>,
}

pub trait LoadJSONConfiguration {
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let path = Self::get_path();

        let config = if path.exists() {
            let file_content = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {:?}", path));

            serde_json::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {:?}. Reason: {}. This is probably caused by a config update. Just delete the old config and restart.",
                    path, err
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(path, serde_json::to_string_pretty(&content).unwrap()) {
                eprintln!(
                    "Couldn't write default config to {:?}. Reason: {}. This is probably caused by a config update. Just delete the old config and restart.",
                    path, err
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
                warn!(
                    "Couldn't serialize operator config to {:?}. Reason: {}",
                    path, err
                );
                return;
            }
        };

        if let Err(err) = fs::write(path, content) {
            warn!(
                "Couldn't write operator config to {:?}. Reason: {}",
                path, err
            );
        }
    }
}

impl LoadJSONConfiguration for OperatorConfig {
    fn get_path() -> &'static Path {
        Path::new("ops.json")
    }
    fn validate(&self) {
        // TODO: Validate the operator configuration
    }
}


impl SaveJSONConfiguration for OperatorConfig {}