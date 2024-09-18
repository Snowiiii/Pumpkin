use lazy_static::lazy_static;

use crate::chunk::ChunkData;

#[derive(Clone)]
pub enum GenerationState {
    Empty,
    StructureStart,
    StructureRef,
    Biome,
    Noise,
    Surface,
    Carver,
    Feature,
    InitLight,
    Light,
    Spawn,
    Full,
}

pub struct Chunk {
    data: ChunkData,
    state: GenerationState,
}

pub struct GenerationShapeConfig {
    y_min: i32,
    height: i32,
    horizontal: i32,
    vertical: i32,
}

//Bits avaliable to encode y-pos
pub const SIZE_BITS_Y: i32 = 12;
pub const MAX_HEIGHT: i32 = (1 << SIZE_BITS_Y) - 32;
pub const MAX_COLUMN_HEIGHT: i32 = (MAX_HEIGHT >> 1) - 1;
pub const MIN_HEIGHT: i32 = MAX_COLUMN_HEIGHT - MAX_HEIGHT + 1;

impl GenerationShapeConfig {
    fn new(y_min: i32, height: i32, horizontal: i32, vertical: i32) -> Self {
        if (y_min + height) > (MAX_COLUMN_HEIGHT + 1) {
            panic!("Cannot be higher than max column height");
        } else if height % 16 != 0 {
            panic!("Height must be a multiple of 16");
        } else if y_min % 16 != 0 {
            panic!("Y min must be a multiple of 16");
        }
        Self {
            y_min,
            height,
            horizontal,
            vertical,
        }
    }
}

lazy_static! {
    pub static ref surface_config: GenerationShapeConfig =
        GenerationShapeConfig::new(-64, 384, 1, 2);
}
