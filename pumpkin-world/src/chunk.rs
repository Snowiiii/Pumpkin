use std::cmp::max;
use std::collections::HashMap;

use fastnbt::LongArray;
use serde::{Deserialize, Serialize};

use crate::{
    block::block_registry::BlockId,
    coordinates::{ChunkCoordinates, ChunkRelativeBlockCoordinates},
    level::WorldError,
    WORLD_HEIGHT,
};

pub struct ChunkData {
    pub blocks: Box<[BlockId; 16 * 16 * WORLD_HEIGHT]>,
    pub position: ChunkCoordinates,
    pub heightmaps: ChunkHeightmaps,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    name: String,
    properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    data: Option<LongArray>,
    palette: Vec<PaletteEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    motion_blocking: LongArray,
    world_surface: LongArray,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i32,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ChunkNbt {
    #[serde(rename = "DataVersion")]
    data_version: usize,
    sections: Vec<ChunkSection>,
    #[serde(rename = "Heightmaps")]
    heightmaps: ChunkHeightmaps,
}

impl ChunkData {
    pub fn from_bytes(chunk_data: Vec<u8>, at: ChunkCoordinates) -> Result<Self, WorldError> {
        let chunk_data = match fastnbt::from_bytes::<ChunkNbt>(chunk_data.as_slice()) {
            Ok(v) => v,
            Err(err) => return Err(WorldError::ErrorDeserializingChunk(err.to_string())),
        };

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut blocks = Box::new([BlockId::default(); 16 * 16 * WORLD_HEIGHT]);
        let mut block_index = 0; // which block we're currently at

        for section in chunk_data.sections.into_iter() {
            let block_states = match section.block_states {
                Some(states) => states,
                None => continue, // TODO @lukas0008 this should instead fill all blocks with the only element of the palette
            };

            let palette = block_states
                .palette
                .iter()
                .map(|entry| BlockId::new(&entry.name, entry.properties.as_ref()))
                .collect::<Result<Vec<_>, _>>()?;

            let block_data = match block_states.data {
                None => {
                    // We skipped placing an empty subchunk.
                    // We need to increase the y coordinate of the next subchunk being placed.
                    block_index += 16 * 16 * 16;
                    continue;
                }
                Some(d) => d,
            }
            .into_inner();

            // How many bits each block has in one of the pallete u64s
            let block_bit_size = {
                let size = 64 - (palette.len() as i64 - 1).leading_zeros();
                max(4, size)
            };
            // How many blocks there are in one of the palletes u64s
            let blocks_in_pallete = 64 / block_bit_size;

            let mask = (1 << block_bit_size) - 1;
            'block_loop: for block in block_data.iter() {
                for i in 0..blocks_in_pallete {
                    let index = (block >> (i * block_bit_size)) & mask;
                    let block = palette[index as usize];

                    blocks[block_index] = block;
                    block_index += 1;

                    // if `SUBCHUNK_VOLUME `is not divisible by `blocks_in_pallete` the block_data
                    // can sometimes spill into other subchunks. We avoid that by aborting early
                    if (block_index % (16 * 16 * 16)) == 0 {
                        break 'block_loop;
                    }
                }
            }
        }

        Ok(ChunkData {
            blocks,
            position: at,
            heightmaps: chunk_data.heightmaps,
        })
    }
    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(
        &mut self,
        at: ChunkRelativeBlockCoordinates,
        block_id: BlockId,
    ) -> Result<BlockId, WorldError> {
        Ok(std::mem::replace(
            &mut self.blocks
                [(at.y.get_absolute() * 16 * 16 + *at.z as u16 * 16 + *at.x as u16) as usize],
            block_id,
        ))
    }
}
