use pumpkin_core::math::position::WorldPosition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u16)]
pub enum SignType {
    OakSign = 194,
    SpruceSign = 195,
    BirchSign = 196,
    AcaciaSign = 197,
    CherrySign = 198,
    JungleSign = 199,
    DarkOakSign = 200,
    PaleOakSign = 201,
    MangroveSign = 202,
    BambooSign = 203,

    OakWallSign = 208,
    SpruceWallSign = 209,
    BirchWallSign = 210,
    AcaciaWallSign = 211,
    CherryWallSign = 212,
    JungleWallSign = 213,
    DarkOakWallSign = 214,
    PaleOakWallSign = 215,
    MangroveWallSign = 216,
    BambooWallSign = 217,

    OakHangingSign = 218,
    SpruceHangingSign = 219,
    BirchHangingSign = 220,
    AcaciaHangingSign = 221,
    CherryHangingSign = 222,
    JungleHangingSign = 223,
    DarkOakHangingSign = 224,
    PaleOakHangingSign = 225,
    CrimsonHangingSign = 226,
    WarpedHangingSign = 227,
    MangroveHangingSign = 228,
    BambooHangingSign = 229,

    OakWallHangingSign = 230,
    SprucWalleHangingSign = 231,
    BirchWallHangingSign = 232,
    AcaciaWallHangingSign = 233,
    CherryWallHangingSign = 234,
    JungleWallHangingSign = 235,
    DarkOakWallHangingSign = 236,
    PaleOakWallHangingSign = 237,
    CrimsonWallHangingSign = 238,
    WarpedWallHangingSign = 239,
    MangroveWallHangingSign = 240,
    BambooWallHangingSign = 241,

    CrimsonSign = 849,
    WarpedSign = 850,
    CrimsonWallSign = 851,
    WarpedWallSign = 852,
}

impl SignType {
    pub fn iter() -> std::slice::Iter<'static, u16> {
        [
            SignType::OakSign as u16,
            SignType::SpruceSign as u16,
            SignType::BirchSign as u16,
            SignType::BirchSign as u16,
            SignType::AcaciaSign as u16,
            SignType::CherrySign as u16,
            SignType::CherrySign as u16,
            SignType::JungleSign as u16,
            SignType::DarkOakSign as u16,
            SignType::PaleOakSign as u16,
            SignType::MangroveSign as u16,
            SignType::BambooSign as u16,
            SignType::OakWallSign as u16,
            SignType::SpruceWallSign as u16,
            SignType::BirchWallSign as u16,
            SignType::BirchWallSign as u16,
            SignType::AcaciaWallSign as u16,
            SignType::CherryWallSign as u16,
            SignType::CherryWallSign as u16,
            SignType::JungleWallSign as u16,
            SignType::DarkOakWallSign as u16,
            SignType::PaleOakWallSign as u16,
            SignType::MangroveWallSign as u16,
            SignType::BambooWallSign as u16,
            SignType::OakHangingSign as u16,
            SignType::SpruceHangingSign as u16,
            SignType::BirchHangingSign as u16,
            SignType::BirchHangingSign as u16,
            SignType::AcaciaHangingSign as u16,
            SignType::CherryHangingSign as u16,
            SignType::CherryHangingSign as u16,
            SignType::JungleHangingSign as u16,
            SignType::DarkOakHangingSign as u16,
            SignType::PaleOakHangingSign as u16,
            SignType::MangroveHangingSign as u16,
            SignType::BambooHangingSign as u16,
        ]
        .iter()
    }
}

// NBT data structure
#[derive(Serialize, Deserialize)]
pub struct Sign {
    id: String,
    is_waxed: u8,
    x: i32,
    y: i32,
    z: i32,
    front_text: Text,
    back_text: Text,
}

#[derive(Serialize, Deserialize)]
struct Text {
    hash_glowing_text: u8,
    color: String,
    messages: Vec<String>,
}

impl Text {
    fn new(messages: Vec<String>) -> Self {
        Self {
            hash_glowing_text: 0,
            color: "black".to_string(),
            messages,
        }
    }
}

impl Sign {
    pub fn new(location: WorldPosition, is_front: bool, messages: &[String]) -> Self {
        Self {
            id: "minecraft:sign".to_string(),
            is_waxed: 0,
            x: location.0.x,
            y: location.0.y,
            z: location.0.z,
            front_text: Text::new(if is_front { messages.to_vec() } else { vec![] }),
            back_text: Text::new(if !is_front { messages.to_vec() } else { vec![] }),
        }
    }
}
