use core::str;

use fastnbt::SerOpts;
use serde::{Deserialize, Serialize};

// represents a text component
// Reference: https://wiki.vg/Text_formatting#Text_components
#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
}

impl Text {
    pub fn encode(&self) -> Vec<u8> {
        fastnbt::to_bytes_with_opts(&self, SerOpts::network_nbt()).unwrap()
    }
}
