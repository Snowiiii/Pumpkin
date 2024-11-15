use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Deserialize, Debug, Clone)]
pub struct Sound {
    pub name: String,
    pub id: u16,
}

pub static SOUNDS: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/sounds.json"))
        .expect("Could not parse sounds.json registry.")
});

#[derive(Debug, Copy, Clone)]
pub enum SoundCategory {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

pub static SOUND_CATEGORIES: LazyLock<HashMap<String, SoundCategory>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("master".to_string(), SoundCategory::Master);
    map.insert("music".to_string(), SoundCategory::Music);
    map.insert("records".to_string(), SoundCategory::Records);
    map.insert("weather".to_string(), SoundCategory::Weather);
    map.insert("blocks".to_string(), SoundCategory::Blocks);
    map.insert("hostile".to_string(), SoundCategory::Hostile);
    map.insert("neutral".to_string(), SoundCategory::Neutral);
    map.insert("players".to_string(), SoundCategory::Players);
    map.insert("ambient".to_string(), SoundCategory::Ambient);
    map.insert("voice".to_string(), SoundCategory::Voice);
    map
});

impl FromStr for SoundCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SOUND_CATEGORIES.get(s).cloned().ok_or(())
    }
}
