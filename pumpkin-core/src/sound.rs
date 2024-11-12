use std::str::FromStr;

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
