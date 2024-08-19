use std::collections::HashMap;

use fastnbt::LongArray;

use crate::{world::WorldError, WORLD_HEIGHT};

pub struct ChunkData {
    pub blocks: Box<[i32; 16 * 16 * WORLD_HEIGHT]>,
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
    data: Option<LongArray>,
    palette: Vec<PaletteEntry>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    motion_blocking: LongArray,
    world_surface: LongArray,
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i32,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
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
                    crate::block::block_registry::block_id_and_properties_to_block_state_id(
                        &entry.name,
                        entry.properties.as_ref(),
                    )
                    .map(|v| v as i32)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let block_data = match block_states.data {
                None => continue,
                Some(d) => d,
            }
            .into_inner();
            let block_size = {
                let size = 64 - (palette.len() as i64 - 1).leading_zeros();
                if size >= 4 {
                    size
                } else {
                    4
                }
            };

            let mask = (1 << block_size) - 1;
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

        Ok(ChunkData {
            blocks,
            position: at,
            heightmaps: chunk_data.heightmaps,
        })
    }
}
