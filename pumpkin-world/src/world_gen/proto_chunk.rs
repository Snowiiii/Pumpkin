use pumpkin_core::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    block::BlockState,
    world_gen::{
        chunk_noise::CHUNK_DIM,
        generation_shapes::GenerationShape,
        noise::{config::NoiseConfig, router::OVERWORLD_NOISE_ROUTER},
        positions::chunk_pos,
    },
};

use super::{
    aquifer_sampler::{FluidLevel, FluidLevelSampler, FluidLevelSamplerImpl},
    chunk_noise::{ChunkNoiseGenerator, LAVA_BLOCK, STONE_BLOCK, WATER_BLOCK},
    positions::chunk_pos::{start_block_x, start_block_z},
};

pub struct StandardChunkFluidLevelSampler {
    top_fluid: FluidLevel,
    bottom_fluid: FluidLevel,
    bottom_y: i32,
}

impl StandardChunkFluidLevelSampler {
    pub fn new(top_fluid: FluidLevel, bottom_fluid: FluidLevel) -> Self {
        let bottom_y = top_fluid
            .max_y_exclusive()
            .min(bottom_fluid.max_y_exclusive());
        Self {
            top_fluid,
            bottom_fluid,
            bottom_y,
        }
    }
}

impl FluidLevelSamplerImpl for StandardChunkFluidLevelSampler {
    fn get_fluid_level(&self, _x: i32, y: i32, _z: i32) -> FluidLevel {
        if y < self.bottom_y {
            self.bottom_fluid.clone()
        } else {
            self.top_fluid.clone()
        }
    }
}

pub struct ProtoChunk {
    chunk_pos: Vector2<i32>,
    sampler: ChunkNoiseGenerator,
    // These are local positions
    flat_block_map: Vec<BlockState>,
    // may want to use chunk status
}

impl ProtoChunk {
    pub fn new(chunk_pos: Vector2<i32>, seed: u64) -> Self {
        // TODO: Don't hardcode these

        let base_router = &OVERWORLD_NOISE_ROUTER;

        let generation_shape = GenerationShape::SURFACE;
        let config = NoiseConfig::new(seed, base_router);

        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();

        // TODO: Customize these
        let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
            FluidLevel::new(63, WATER_BLOCK),
            FluidLevel::new(-54, LAVA_BLOCK),
        ));

        let height = generation_shape.height() as usize;
        let sampler = ChunkNoiseGenerator::new(
            horizontal_cell_count,
            chunk_pos::start_block_x(&chunk_pos),
            chunk_pos::start_block_z(&chunk_pos),
            generation_shape,
            &config,
            sampler,
            true,
            true,
        );

        Self {
            chunk_pos,
            sampler,
            flat_block_map: vec![BlockState::AIR; CHUNK_DIM as usize * CHUNK_DIM as usize * height],
        }
    }

    #[inline]
    fn local_pos_to_index(&self, local_pos: &Vector3<i32>) -> usize {
        #[cfg(debug_assertions)]
        {
            assert!(local_pos.x >= 0 && local_pos.x <= 15);
            assert!(local_pos.y < self.sampler.height() as i32 && local_pos.y >= 0);
            assert!(local_pos.z >= 0 && local_pos.z <= 15);
        }
        self.sampler.height() as usize * CHUNK_DIM as usize * local_pos.x as usize
            + CHUNK_DIM as usize * local_pos.y as usize
            + local_pos.z as usize
    }

    #[inline]
    pub fn get_block_state(&self, local_pos: &Vector3<i32>) -> BlockState {
        let local_pos = Vector3::new(
            local_pos.x & 15,
            local_pos.y - self.sampler.min_y() as i32,
            local_pos.z & 15,
        );
        if local_pos.y < 0 || local_pos.y >= self.sampler.height() as i32 {
            BlockState::AIR
        } else {
            self.flat_block_map[self.local_pos_to_index(&local_pos)]
        }
    }

    pub fn populate_noise(&mut self) {
        let horizontal_cell_block_count = self.sampler.horizontal_cell_block_count();
        let vertical_cell_block_count = self.sampler.vertical_cell_block_count();

        let horizontal_cells = CHUNK_DIM / horizontal_cell_block_count;

        let min_y = self.sampler.min_y();
        let minimum_cell_y = min_y / vertical_cell_block_count as i8;
        let cell_height = self.sampler.height() / vertical_cell_block_count as u16;

        // Safety
        //
        // Owned density functions are only invoked in `sampler.sample_block_state`:
        //     - Everything in this block is ran from the same thread
        //     - All unsafe functions are encapsulated and no mutable references are leaked
        unsafe {
            self.sampler.sample_start_density();
            for cell_x in 0..horizontal_cells {
                self.sampler.sample_end_density(cell_x);

                for cell_z in 0..horizontal_cells {
                    for cell_y in (0..cell_height).rev() {
                        self.sampler.on_sampled_cell_corners(cell_y, cell_z);
                        for local_y in (0..vertical_cell_block_count).rev() {
                            let block_y = (minimum_cell_y as i32 + cell_y as i32)
                                * vertical_cell_block_count as i32
                                + local_y as i32;
                            let delta_y = local_y as f64 / vertical_cell_block_count as f64;
                            self.sampler.interpolate_y(block_y, delta_y);

                            for local_x in 0..horizontal_cell_block_count {
                                let block_x = self.start_block_x()
                                    + cell_x as i32 * horizontal_cell_block_count as i32
                                    + local_x as i32;
                                let delta_x = local_x as f64 / horizontal_cell_block_count as f64;
                                self.sampler.interpolate_x(block_x, delta_x);

                                for local_z in 0..horizontal_cell_block_count {
                                    let block_z = self.start_block_z()
                                        + cell_z as i32 * horizontal_cell_block_count as i32
                                        + local_z as i32;
                                    let delta_z =
                                        local_z as f64 / horizontal_cell_block_count as f64;
                                    self.sampler.interpolate_z(block_z, delta_z);

                                    // TODO: Change default block
                                    let block_state =
                                        self.sampler.sample_block_state().unwrap_or(STONE_BLOCK);
                                    //log::debug!("Sampled block state in {:?}", inst.elapsed());

                                    let local_pos = Vector3 {
                                        x: block_x & 15,
                                        y: block_y - min_y as i32,
                                        z: block_z & 15,
                                    };

                                    #[cfg(debug_assertions)]
                                    {
                                        assert!(local_pos.x < 16 && local_pos.x >= 0);
                                        assert!(
                                            local_pos.y < self.sampler.height() as i32
                                                && local_pos.y >= 0
                                        );
                                        assert!(local_pos.z < 16 && local_pos.z >= 0);
                                    }
                                    let index = self.local_pos_to_index(&local_pos);
                                    self.flat_block_map[index] = block_state;
                                }
                            }
                        }
                    }
                }

                self.sampler.swap_buffers();
            }
        }
        self.sampler.stop_interpolation();
    }

    fn start_cell_x(&self) -> i32 {
        self.start_block_x() / self.sampler.horizontal_cell_block_count() as i32
    }

    fn start_cell_z(&self) -> i32 {
        self.start_block_z() / self.sampler.horizontal_cell_block_count() as i32
    }

    fn start_block_x(&self) -> i32 {
        start_block_x(&self.chunk_pos)
    }

    fn start_block_z(&self) -> i32 {
        start_block_z(&self.chunk_pos)
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    use pumpkin_core::math::vector2::Vector2;

    use crate::read_data_from_file;

    use super::ProtoChunk;

    #[test]
    fn test_no_blend_no_beard() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_0_0.chunk");
        let mut chunk = ProtoChunk::new(Vector2::new(0, 0), 0);
        chunk.populate_noise();
        assert_eq!(
            expected_data,
            chunk
                .flat_block_map
                .into_iter()
                .map(|state| state.state_id)
                .collect::<Vec<u16>>()
        );
    }

    #[test]
    fn test_no_blend_no_beard_aquifer() {
        let expected_data: Vec<u16> =
            read_data_from_file!("../../assets/no_blend_no_beard_7_4.chunk");
        let mut chunk = ProtoChunk::new(Vector2::new(7, 4), 0);
        chunk.populate_noise();

        assert_eq!(
            expected_data,
            chunk
                .flat_block_map
                .into_iter()
                .map(|state| state.state_id)
                .collect::<Vec<u16>>()
        );
    }
}
