use fastnbt::{nbt, SerOpts};

pub const BLOCKS_AND_BIOMES: [u8; 2000] = [0x80; 2000];
pub const SKY_LIGHT_ARRAYS: [FixedArray<u8, 2048>; 26] = [FixedArray([0xff; 2048]); 26];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct FixedArray<T, const N: usize>(pub [T; N]);

pub struct TestChunk {
    pub heightmap: Vec<u8>,
}

impl TestChunk {
    pub fn new() -> Self {
        let bytes = fastnbt::to_bytes(&nbt!({"MOTION_BLOCKING": [L; 123, 256]})).unwrap();

        Self { heightmap: bytes }
    }
}
