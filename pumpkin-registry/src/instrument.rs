use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    sound_event: String,
    use_duration: f32,
    range: f32,
    //  description: TextComponent<'static>,
}
