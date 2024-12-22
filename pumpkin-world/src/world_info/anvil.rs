use std::{fs::OpenOptions, io::Read};

use flate2::read::GzDecoder;
use serde::Deserialize;

use crate::level::SaveFile;

use super::{WorldInfo, WorldInfoError, WorldInfoReader};

pub struct AnvilInfoReader {}

impl AnvilInfoReader {
    pub fn new() -> Self {
        Self {}
    }
}

impl WorldInfoReader for AnvilInfoReader {
    fn read_world_info(&self, save_file: &SaveFile) -> Result<WorldInfo, WorldInfoError> {
        let path = save_file.root_folder.join("level.dat");

        let mut world_info_file = OpenOptions::new().read(true).open(path)?;

        let mut buffer = Vec::new();
        world_info_file.read_to_end(&mut buffer)?;

        let mut decoder = GzDecoder::new(&buffer[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let info = fastnbt::from_bytes::<LevelDat>(&decompressed_data)
            .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

        Ok(WorldInfo {
            seed: info.data.world_gen_settings.seed,
        })
    }
}


impl Default for AnvilInfoReader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub struct LevelDat {
    // No idea why its formatted like this
    #[serde(rename = "Data")]
    pub data: WorldData,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorldData {
    pub world_gen_settings: WorldGenSettings,
    // TODO: Implement the rest of the fields
    // Fields below this comment are being deserialized, but are not being used
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
}

#[derive(Deserialize)]
pub struct WorldGenSettings {
    pub seed: i64,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        level::SaveFile,
        world_info::{anvil::AnvilInfoReader, WorldInfo, WorldInfoReader},
    };

    #[test]
    fn test_level_dat_reading() {
        let world_info = AnvilInfoReader::new();
        let root_folder = PathBuf::from("test-files").join("sample-1");
        let save_file = SaveFile {
            root_folder: root_folder.clone(),
            region_folder: root_folder,
        };
        let expected = WorldInfo {
            seed: -79717552349559436,
        };
        let info = world_info.read_world_info(&save_file).unwrap();

        assert_eq!(info, expected);
    }
}
