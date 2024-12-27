use std::{path::Path, sync::LazyLock};

use pumpkin_config::op;
use serde::{Deserialize, Serialize};

use super::{LoadJSONConfiguration, SaveJSONConfiguration};

pub static OPERATOR_CONFIG: LazyLock<tokio::sync::RwLock<OperatorConfig>> =
    LazyLock::new(|| tokio::sync::RwLock::new(OperatorConfig::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct OperatorConfig {
    pub ops: Vec<op::Op>,
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
