#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct ServerLinksConfig {
    pub enabled: bool,
    pub bug_report: String,
    pub support: String,
    pub status: String,
    pub feedback: String,
    pub community: String,
    pub website: String,
    pub forums: String,
    pub news: String,
    pub announcements: String,
    pub custom: HashMap<String, String>,
}

impl Default for ServerLinksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bug_report: "https://github.com/Snowiiii/Pumpkin/issues".to_string(),
            support: "".to_string(),
            status: "".to_string(),
            feedback: "".to_string(),
            community: "".to_string(),
            website: "".to_string(),
            forums: "".to_string(),
            news: "".to_string(),
            announcements: "".to_string(),
            custom: Default::default(),
        }
    }
}
