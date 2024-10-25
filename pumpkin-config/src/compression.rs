use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
/// Packet compression
pub struct CompressionConfig {
    /// Wether compression is enabled
    #[serde_inline_default(true)]
    pub enabled: bool,
    #[serde(flatten)]
    #[serde(default)]
    pub compression_info: CompressionInfo,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, Clone)]
/// We have this in a Seperate struct so we can use it outside of the Config
pub struct CompressionInfo {
    /// The compression threshold used when compression is enabled
    #[serde_inline_default(256)]
    pub threshold: u32,
    /// A value between 0..9
    /// 1 = Optimize for the best speed of encoding.
    /// 9 = Optimize for the size of data being encoded.
    #[serde_inline_default(4)]
    pub level: u32,
}

impl Default for CompressionInfo {
    fn default() -> Self {
        Self {
            threshold: 256,
            level: 4,
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_info: Default::default(),
        }
    }
}
