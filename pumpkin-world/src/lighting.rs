use std::collections::VecDeque;

use fastnbt::LongArray;

use crate::{
    chunk::{ChunkBlocks, CHUNK_AREA, CHUNK_VOLUME, SUBCHUNK_VOLUME},
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    WORLD_HEIGHT,
};

#[derive(Clone, Copy, Debug)]
pub struct BlockLight(u8);

impl Default for BlockLight {
    fn default() -> Self {
        Self(0)
    }
}

impl BlockLight {
    const SKYLIGHT_MASK: u8 = 0x0F;
    const BLOCKLIGHT_MASK: u8 = 0xF0;

    pub fn set_skylight(self, value: u8) -> Self {
        BlockLight(self.0 | value & Self::SKYLIGHT_MASK)
    }

    pub fn set_blocklight(self, value: u8) -> Self {
        BlockLight(self.0 | (value << 4) & Self::BLOCKLIGHT_MASK)
    }

    pub fn get_skylight(&self) -> u8 {
        self.0 & Self::SKYLIGHT_MASK
    }

    pub fn get_blocklight(&self) -> u8 {
        (self.0 & Self::BLOCKLIGHT_MASK) >> 4
    }
}

const LIGHT_VOLUME: usize = CHUNK_VOLUME + 2 * SUBCHUNK_VOLUME;

// TODO: Work out the sub chunks, what is required to be sent
// Only store/send required subchunks
// Propagate over chunk boundaries.

pub struct ChunkLighting<'a, 'b> {
    blocks: &'a ChunkBlocks,
    height_map: &'b LongArray,
    light_data: [BlockLight; LIGHT_VOLUME],
    increase_queue: VecDeque<(Coordinates, u8)>,
}

type Coordinates = (u16, u16, u16);

impl<'a, 'b> ChunkLighting<'a, 'b> {
    pub fn new(blocks: &'a ChunkBlocks, height_map: &'b LongArray) -> Self {
        Self {
            blocks,
            height_map,
            light_data: [BlockLight::default(); LIGHT_VOLUME],
            increase_queue: VecDeque::new(),
        }
    }

    pub fn generate_initial_lighting(mut self) -> [BlockLight; LIGHT_VOLUME] {
        // The heightmaps are not actually set to the max height of a non air block
        // will manually check every column instead
        //
        // for (i, height) in height_map.to_vec().iter().enumerate() {
        //     let x = i as u16 % 16;
        //     let z = i as u16 / 16;
        //     let lowest_y = *height as u16 + 1;
        //
        //     self.set_light_source((x, lowest_y, z), 15);
        // }

        let mut skylight_sources = Vec::new();
        for i in 0..CHUNK_AREA {
            let x = i as u16 % 16;
            let z = i as u16 / 16;

            let mut lowest_y = WORLD_HEIGHT as u16 + 16;
            for y in 1..=(WORLD_HEIGHT as u16) {
                let y = (WORLD_HEIGHT as u16) - y + 16;
                if !self.is_air((x, y, z)) {
                    break;
                }
                lowest_y = y;

                let light_index = Self::coords_to_light_index((x, y, z));
                // self.light_data[light_index] = BlockLight(0).set_skylight(15);
            }

            skylight_sources.push((x, lowest_y, z));
        }

        let highest_source = skylight_sources.iter().map(|s| s.1).max().unwrap();

        let max_subchunk = highest_source / 16;
        let min_subchunk = 1;

        for source in skylight_sources {
            self.set_skylight_source(source, 15);
        }

        self.light_data
    }

    fn set_skylight_source(&mut self, coordinates: Coordinates, level: u8) {
        println!("{}", coordinates.1 % 16);
        for i in coordinates.1 % 16 + 1..32 {
            let base = (coordinates.1 / 16) * 16;
            let light_index = Self::coords_to_light_index((coordinates.0, base + i, coordinates.2));
            let current_light_level = self.light_data[light_index];
            self.light_data[light_index] = current_light_level.set_skylight(level);
        }

        let light_index = Self::coords_to_light_index(coordinates);

        let current_light_level = self.light_data[light_index];
        if level <= current_light_level.get_skylight() {
            return;
        }

        self.light_data[light_index] = current_light_level.set_skylight(level);

        self.increase_queue.push_back((coordinates, level));

        self.propogate_increase();
    }

    fn directions(coordinates: Coordinates) -> Vec<Coordinates> {
        let mut directions = vec![];

        if coordinates.0 < 15 {
            directions.push((coordinates.0 + 1, coordinates.1, coordinates.2));
        }
        if coordinates.0 > 0 {
            directions.push((coordinates.0 - 1, coordinates.1, coordinates.2));
        }
        if coordinates.1 < (WORLD_HEIGHT as u16 + 32) {
            directions.push((coordinates.0, coordinates.1 + 1, coordinates.2));
        }
        if coordinates.1 > 0 {
            directions.push((coordinates.0, coordinates.1 - 1, coordinates.2));
        }
        if coordinates.2 < 15 {
            directions.push((coordinates.0, coordinates.1, coordinates.2 + 1));
        }
        if coordinates.2 > 0 {
            directions.push((coordinates.0, coordinates.1, coordinates.2 - 1));
        }

        directions
    }

    fn coords_to_light_index(coordinates: Coordinates) -> usize {
        coordinates.1 as usize * CHUNK_AREA + coordinates.2 as usize * 16 + coordinates.0 as usize
    }
    fn coords_to_block_coords(coordinates: Coordinates) -> ChunkRelativeBlockCoordinates {
        assert!(coordinates.1 > 15);
        assert!(coordinates.1 as usize <= 16 + WORLD_HEIGHT);
        ChunkRelativeBlockCoordinates {
            x: coordinates.0.into(),
            y: Height::from_absolute(coordinates.1 - 16),
            z: coordinates.2.into(),
        }
    }

    fn is_air(&self, coordinates: Coordinates) -> bool {
        if coordinates.1 < 16 || coordinates.1 as usize >= 16 + WORLD_HEIGHT {
            return true;
        }
        self.blocks
            .get_block(Self::coords_to_block_coords(coordinates))
            .is_air()
    }

    fn propogate_increase(&mut self) {
        while let Some((coordinates, incoming_light_level)) = self.increase_queue.pop_front() {
            for neighbor_coordinates in Self::directions(coordinates) {
                let light_index = Self::coords_to_light_index(neighbor_coordinates);

                let current_light_level = self.light_data[light_index];
                if current_light_level.get_skylight() >= incoming_light_level - 1 {
                    continue;
                }

                let is_air = self.is_air(neighbor_coordinates);
                let target_level = if is_air { incoming_light_level - 1 } else { 0 };

                if target_level > current_light_level.get_skylight() {
                    self.light_data[light_index] = current_light_level.set_skylight(target_level);
                    self.increase_queue
                        .push_back((neighbor_coordinates, target_level));
                }
            }
        }
    }
}
