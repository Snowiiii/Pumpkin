use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
// Packet compression
pub struct CompressionConfig {
    /// Is compression enabled ?
    pub enabled: bool,
    /// The compression threshold used when compression is enabled
    pub compression_threshold: u32,
    /// A value between 0..9
    /// 1 = Optimize for the best speed of encoding.
    /// 9 = Optimize for the size of data being encoded.
    pub compression_level: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_threshold: 256,
            compression_level: 4,
        }
    }
}
