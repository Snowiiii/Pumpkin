use pumpkin_core::math::position::WorldPosition;
use serde::{Deserialize, Serialize};

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
