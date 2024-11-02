use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukeboxSong {
    sound_event: String,
    // description: TextComponent<'static>,
    length_in_seconds: f32,
    comparator_output: u32,
}
