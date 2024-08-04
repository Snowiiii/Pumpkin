use std::{fs::File, path::Path};

use fastanvil::Region;

pub mod chunk;

pub struct World {
    pub region: Region<File>,
}

impl World {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let file = std::fs::File::open(path).unwrap();

        let region = Region::from_stream(file).unwrap();
        Self { region }
    }

    /*   pub fn load_chunk(&mut self, x: usize, y: usize) -> Chunk {
        let data = self.region.read_chunk(x, y).unwrap().unwrap();

        let complete_chunk = complete::Chunk::from_bytes(&data).unwrap();

    }
    */
}
