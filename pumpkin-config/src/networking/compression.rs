use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
/// Packet compression
pub struct CompressionConfig {
    /// Wether compression is enabled
    pub enabled: bool,
    #[serde(flatten)]
    pub compression_info: CompressionInfo,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_info: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
/// We have this in a Separate struct so we can use it outside of the Config
pub struct CompressionInfo {
    /// The compression threshold used when compression is enabled
    pub threshold: u32,
    /// A value between 0..9
    /// 1 = Optimize for the best speed of encoding.
    /// 9 = Optimize for the size of data being encoded.
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
