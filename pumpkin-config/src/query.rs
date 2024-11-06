use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize, Default)]
pub struct QueryConfig {
    #[serde_inline_default(false)]
    pub enabled: bool,
    // Optional so if not specified the port server is running on will be used
    #[serde_inline_default(None)]
    pub port: Option<u16>,
}
