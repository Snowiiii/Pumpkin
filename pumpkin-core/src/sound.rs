use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Deserialize, Debug)]
pub struct Sound {
    name: String,
    id: u16,
}

pub static SOUNDS: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<Sound>>(include_str!("../../assets/sounds.json"))
        .expect("Could not parse sounds.json registry.")
        .into_iter()
        .map(|val| (val.name, val.id))
        .collect()
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

impl FromStr for SoundCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "master" => Ok(Self::Master),
            "music" => Ok(Self::Music),
            "records" => Ok(Self::Records),
            "weather" => Ok(Self::Weather),
            "blocks" => Ok(Self::Blocks),
            "hostile" => Ok(Self::Hostile),
            "neutral" => Ok(Self::Neutral),
            "players" => Ok(Self::Players),
            "ambient" => Ok(Self::Ambient),
            "voice" => Ok(Self::Voice),
            _ => {
                log::info!("Unexpected SoundCategory {}", s);
                Err(())
            }
        }
    }
}
