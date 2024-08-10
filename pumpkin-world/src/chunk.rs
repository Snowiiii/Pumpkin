// use fastnbt::nbt;

// pub const BLOCKS_AND_BIOMES: [u8; 2000] = [0x80; 2000];
// pub const SKY_LIGHT_ARRAYS: [FixedArray<u8, 2048>; 26] = [FixedArray([0xff; 2048]); 26];

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// #[repr(transparent)]
// pub struct FixedArray<T, const N: usize>(pub [T; N]);

// pub struct TestChunk {
//     pub heightmap: Vec<u8>,
// }

// impl Default for TestChunk {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl TestChunk {
//     pub fn new() -> Self {
//         let bytes = fastnbt::to_bytes(&nbt!({"MOTION_BLOCKING": [L; 123, 256]})).unwrap();

//         Self { heightmap: bytes }
//     }
// }

use std::{collections::HashMap, io::Write};

use fastnbt::LongArray;
use itertools::Itertools;

use crate::{world::WorldError, WORLD_HEIGHT};

pub struct ChunkData {
    pub blocks: Box<[i64; 16 * 16 * WORLD_HEIGHT]>,
    pub position: (i32, i32),
    pub heightmaps: ChunkHeightmaps,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    name: String,
    properties: Option<HashMap<String, String>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    data: Option<LongArray>, // TODO: see if you can use u32 here
    palette: Vec<PaletteEntry>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    motion_blocking: LongArray,
    world_surface: LongArray,
}

#[derive(serde::Deserialize, Debug)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i32,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(serde::Deserialize, Debug)]
struct ChunkNbt {
    #[serde(rename = "DataVersion")]
    data_version: usize,
    sections: Vec<ChunkSection>,
    #[serde(rename = "Heightmaps")]
    heightmaps: ChunkHeightmaps,
}

impl ChunkData {
    pub fn from_bytes(chunk_data: Vec<u8>, at: (i32, i32)) -> Result<Self, WorldError> {
        let chunk_data = match fastnbt::from_bytes::<ChunkNbt>(chunk_data.as_slice()) {
            Ok(v) => v,
            Err(err) => return Err(WorldError::ErrorDeserializingChunk(err.to_string())),
        };

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut blocks = Box::new([0; 16 * 16 * WORLD_HEIGHT]);

        for (k, section) in chunk_data.sections.into_iter().enumerate() {
            let block_states = match section.block_states {
                Some(states) => states,
                None => continue, // this should instead fill all blocks with the only element of the palette
            };
            let palette = block_states
                .palette
                .iter()
                .map(|entry| {
                    crate::block_registry::block_id_and_properties_to_block_state_id(
                        &entry.name,
                        entry.properties.as_ref(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let block_data = match block_states.data {
                None => continue,
                Some(d) => d,
            }
            .into_inner();
            let block_size = {
                let size = 64 - (palette.len() - 1).leading_zeros();
                if size >= 4 {
                    size
                } else {
                    4
                }
            };

            let mask = (1 << block_size) - 1;
            println!("{block_size} {:#08b}", mask);
            println!(
                "{} {}",
                block_size,
                palette.iter().map(|v| v.to_string()).join(",")
            );
            let mut blocks_left = 16 * 16 * 16;
            'block_loop: for (j, block) in block_data.iter().enumerate() {
                for i in 0..64 / block_size {
                    if blocks_left <= 0 {
                        break 'block_loop;
                    }
                    let index = (block >> (i * block_size)) & mask;
                    let block = palette[index as usize];
                    blocks[k * 16 * 16 * 16 + j * ((64 / block_size) as usize) + i as usize] =
                        block;
                    blocks_left -= 1;
                }
            }
        }

        // std::fs::File::create_new("../mafaile.txt")
        //     .unwrap()
        //     .write_all(
        //         blocks
        //             .iter()
        //             .map(|v| v.to_string())
        //             .collect::<Vec<_>>()
        //             .join(",")
        //             .as_bytes(),
        //     );

        if at == (0, 0) {
            println!(
                "[{}]",
                &blocks[0..10000].iter().map(|v| v.to_string()).join(",")
            );
        }
        Ok(ChunkData {
            blocks: blocks,
            position: at,
            heightmaps: chunk_data.heightmaps,
        })
    }
}
