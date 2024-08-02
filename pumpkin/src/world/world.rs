use crate::{entity::player::Player, game::data::GameData};

use super::{chunk::WorldChunk, leveldat::LevelDat};

pub struct World {
    pub players: Vec<Player>,
    pub loaded_chunks: Vec<WorldChunk>,
    pub level_dat: LevelDat,
}

impl World {
    //path is the path to the world and region is the path to the region folder in the world
    pub fn new(path: &str, region: &str) -> Self {
        //load game data
        let game_data = &GameData::default();

        //load some chunks

        let mut loaded_chunks = Vec::new();

        let chunk_1 = WorldChunk::load_chunk(region, 0, 0, game_data);
        let chunk_2 = WorldChunk::load_chunk(region, 0, 1, game_data);
        let chunk_3 = WorldChunk::load_chunk(region, 1, 0, game_data);
        let chunk_4 = WorldChunk::load_chunk(region, 1, 1, game_data);

        loaded_chunks.push(chunk_1);
        loaded_chunks.push(chunk_2);
        loaded_chunks.push(chunk_3);
        loaded_chunks.push(chunk_4);

        Self {
            players: Vec::new(),
            loaded_chunks,
            level_dat: LevelDat::load(format!("{}/level.dat", path).as_str()),
        }
    }

    pub fn get_region_file(x: f32, z: f32) -> String {
        let result: f32 = 7.0 / 32.0;
        //dbg!("{}", result.floor());

        let region_file_name = format!("r.{}.{}.mca", (x / 32.0).floor(), (z / 32.0).floor());
        region_file_name
    }

    pub fn get_intern_coords(minecraft_x: i32, minecraft_z: i32) -> (i32, i32) {
        let result_x = minecraft_x.rem_euclid(32);
        let result_z = minecraft_z.rem_euclid(32);

        (result_x, result_z)
    }
}
