use pumpkin_core::random::RandomDeriver;

use crate::block::BlockState;

use super::{
    chunk_noise::ChunkNoiseState,
    noise::{
        clamped_map,
        density::{component_functions::ComponentReference, NoisePos, NoisePosImpl},
    },
};

pub struct OreVeinSampler {
    vein_toggle: Box<dyn ComponentReference<ChunkNoiseState>>,
    vein_ridged: Box<dyn ComponentReference<ChunkNoiseState>>,
    vein_gap: Box<dyn ComponentReference<ChunkNoiseState>>,
    random_deriver: RandomDeriver,
}

impl OreVeinSampler {
    pub fn new(
        vein_toggle: Box<dyn ComponentReference<ChunkNoiseState>>,
        vein_ridged: Box<dyn ComponentReference<ChunkNoiseState>>,
        vein_gap: Box<dyn ComponentReference<ChunkNoiseState>>,
        random_deriver: RandomDeriver,
    ) -> Self {
        Self {
            vein_toggle,
            vein_ridged,
            vein_gap,
            random_deriver,
        }
    }
}

impl OreVeinSampler {
    pub fn sample(&mut self, pos: &NoisePos, state: &ChunkNoiseState) -> Option<BlockState> {
        let vein_sample = self.vein_toggle.sample_mut(pos, state);
        let vein_type: &VeinType = if vein_sample > 0f64 {
            &vein_type::COPPER
        } else {
            &vein_type::IRON
        };

        let block_y = pos.y();
        let max_to_y = vein_type.max_y - block_y;
        let y_to_min = block_y - vein_type.min_y;
        if (max_to_y >= 0) && (y_to_min >= 0) {
            let closest_to_bound = max_to_y.min(y_to_min);
            let mapped_diff = clamped_map(closest_to_bound as f64, 0f64, 20f64, -0.2f64, 0f64);
            let abs_sample = vein_sample.abs();
            if abs_sample + mapped_diff >= 0.4f32 as f64 {
                let mut random = self.random_deriver.split_pos(pos.x(), block_y, pos.z());
                if random.next_f32() <= 0.7f32 && self.vein_ridged.sample_mut(pos, state) < 0f64 {
                    let clamped_sample = clamped_map(
                        abs_sample,
                        0.4f32 as f64,
                        0.6f32 as f64,
                        0.1f32 as f64,
                        0.3f32 as f64,
                    );

                    return if (random.next_f32() as f64) < clamped_sample
                        && self.vein_gap.sample_mut(pos, state) > (-0.3f32 as f64)
                    {
                        Some(if random.next_f32() < 0.02f32 {
                            vein_type.raw_ore
                        } else {
                            vein_type.ore
                        })
                    } else {
                        Some(vein_type.stone)
                    };
                }
            }
        }
        None
    }
}

pub struct VeinType {
    ore: BlockState,
    raw_ore: BlockState,
    stone: BlockState,
    min_y: i32,
    max_y: i32,
}

// One of the victims of removing compile time blocks
pub mod vein_type {
    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        pub static ref COPPER: VeinType = VeinType {
            ore: BlockState::new("minecraft:copper_ore").unwrap(),
            raw_ore: BlockState::new("minecraft:raw_copper_block").unwrap(),
            stone: BlockState::new("minecraft:granite").unwrap(),
            min_y: 0,
            max_y: 50,
        };
        pub static ref IRON: VeinType = VeinType {
            ore: BlockState::new("minecraft:deepslate_iron_ore").unwrap(),
            raw_ore: BlockState::new("minecraft:raw_iron_block").unwrap(),
            stone: BlockState::new("minecraft:tuff").unwrap(),
            min_y: -60,
            max_y: -8,
        };
        pub static ref MIN_Y: i32 = IRON.min_y;
        pub static ref MAX_Y: i32 = COPPER.max_y;
    }
}
