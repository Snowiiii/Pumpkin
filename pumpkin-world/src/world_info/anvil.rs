use std::{
    fs::OpenOptions,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};

use crate::level::LevelFolder;

use super::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};

const LEVEL_DAT_FILE_NAME: &str = "level.dat";

pub struct AnvilLevelInfo;

impl WorldInfoReader for AnvilLevelInfo {
    fn read_world_info(&self, level_folder: &LevelFolder) -> Result<LevelData, WorldInfoError> {
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);

        let mut world_info_file = OpenOptions::new().read(true).open(path)?;

        let mut buffer = Vec::new();
        world_info_file.read_to_end(&mut buffer)?;

        // try to decompress using GZip
        let mut decoder = GzDecoder::new(&buffer[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let info = fastnbt::from_bytes::<LevelDat>(&decompressed_data)
            .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

        // todo check version

        Ok(info.data)
    }
}

impl WorldInfoWriter for AnvilLevelInfo {
    fn write_world_info(
        &self,
        info: LevelData,
        level_folder: &LevelFolder,
    ) -> Result<(), WorldInfoError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let level = LevelDat {
            data: LevelData {
                allow_commands: info.allow_commands,
                data_version: info.data_version,
                difficulty: info.difficulty,
                world_gen_settings: info.world_gen_settings,
                last_played: since_the_epoch.as_millis() as i64,
                level_name: info.level_name,
                spawn_x: info.spawn_x,
                spawn_y: info.spawn_y,
                spawn_z: info.spawn_z,
                nbt_version: info.nbt_version,
                version: info.version,
            },
        };
        // convert it into nbt
        let nbt = pumpkin_nbt::serializer::to_bytes_unnamed(&level).unwrap();
        // now compress using GZip, TODO: im not sure about the to_vec, but writer is not implemented for BytesMut, see https://github.com/tokio-rs/bytes/pull/478
        let mut encoder = GzEncoder::new(nbt.to_vec(), Compression::best());
        let compressed_data = Vec::new();
        encoder.write_all(&compressed_data)?;

        // open file
        let path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);
        let mut world_info_file = OpenOptions::new().write(true).open(path)?;
        // write compressed data into file
        world_info_file.write_all(&compressed_data).unwrap();

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct LevelDat {
    // This tag contains all the level data.
    #[serde(rename = "Data")]
    pub data: LevelData,
}
