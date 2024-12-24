use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukeboxSong {
    sound_event: String,
    description: Description,
    length_in_seconds: f32,
    comparator_output: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Description {
    translate: String,
}
