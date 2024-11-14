use pumpkin_core::math::position::WorldPosition;
use serde::{Deserialize, Serialize};

// NBT data structure
#[derive(Serialize, Deserialize)]
pub struct Sign {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    front_text: Option<Text>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    back_text: Option<Text>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_waxed: Option<u8>,
    x: i32,
    y: i32,
    z: i32,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Text {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    has_glowing_text: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    messages: [String; 4],
}

impl Text {
    fn new(messages: [String; 4]) -> Self {
        Self {
            has_glowing_text: None,
            color: None,
            messages,
        }
    }
}

impl Sign {
    pub fn new(location: WorldPosition, is_front: bool, messages: [String; 4]) -> Self {
        let formatted_messages = [
            format!("\"{}\"", messages[0]),
            format!("\"{}\"", messages[1]),
            format!("\"{}\"", messages[2]),
            format!("\"{}\"", messages[3]),
        ];

        Self {
            id: "minecraft:sign".to_string(),
            is_waxed: None,
            x: location.0.x,
            y: location.0.y,
            z: location.0.z,
            front_text: if is_front {
                Some(Text::new(formatted_messages.clone()))
            } else {
                None
            },
            back_text: if !is_front {
                Some(Text::new(formatted_messages.clone()))
            } else {
                None
            },
        }
    }
}
