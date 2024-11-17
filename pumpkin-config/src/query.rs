use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct QueryConfig {
    pub enabled: bool,
    // Optional so if not specified the port server is running on will be used
    pub port: Option<u16>,
}
