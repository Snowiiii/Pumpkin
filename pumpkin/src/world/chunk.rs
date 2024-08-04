use std::{borrow::Borrow, collections::HashMap, io::Read};

use fastanvil::{complete, Chunk, Region};

use crate::game::data::GameData;

use super::{block::Block, world::World};

#[derive(Debug)]
pub struct WorldChunk {
    pub blocks: Vec<Block>,
    pub height_map: [i16; 256],
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl WorldChunk {
    //region is the folder with the region files in world it is: world/region, in world_nether it is: world_nether/DIM-1/region for example
    pub fn load_chunk(region: &str, x: i32, z: i32, game_data: &GameData) -> Self {
        let mut blocks: Vec<Block> = Vec::new();

        let region_file = format!("{}/{}", region, World::get_region_file(x as f32, z as f32));
        let intern = World::get_intern_coords(x, z);
        let file = std::fs::File::open(region_file).unwrap();

        let mut region = Region::from_stream(file).unwrap();
        let chunk = region
            .read_chunk(intern.0 as usize, intern.1 as usize)
            .unwrap();
        let complete_chunk = complete::Chunk::from_bytes(&chunk.unwrap()).unwrap();
        let height_map = complete_chunk.heightmap;

        for x in 0..16 {
            for y in -64..320 {
                for z in 0..16 {
                    let chunk_block = complete_chunk.block(x, y, z).unwrap();
                    let biome = complete_chunk.biome(x, y, z).unwrap();

                    //no data = air
                    if chunk_block.name() != "minecraft:air" {
                        let mut block_data = HashMap::new();
                        let description = chunk_block.encoded_description();
                        let desc_split = description.split('|').collect::<Vec<&str>>();
                        if !desc_split.get(1).unwrap().is_empty() {
                            let properties = desc_split.get(1).unwrap();
                            let props = properties.split(',').collect::<Vec<&str>>();
                            for property in props {
                                let split = property.split('=').collect::<Vec<&str>>();
                                let key = split[0].to_string();
                                let value = split[1].to_string();
                                block_data.insert(key, value);
                            }
                        }

                        let block_id = game_data.get_block_id(
                            chunk_block
                                .name()
                                .split(':')
                                .collect::<Vec<&str>>()
                                .get(1)
                                .unwrap()
                                .to_string(),
                        );

                        let block = Block {
                            x: x as i32,
                            y: y as i32,
                            z: z as i32,
                            id: block_id,
                            biome,
                            properties: block_data,
                        };

                        blocks.push(block)
                    }
                }
            }
        }

        Self {
            blocks,
            height_map,
            chunk_x: x,
            chunk_z: z,
        }
    }
}
