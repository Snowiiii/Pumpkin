#![feature(test)]

use std::io::Read;

use flate2::read::GzDecoder;

#[allow(soft_unstable)]

pub struct Testing {}
impl Testing {
    pub fn data_test() {
        dbg!("test hello");
        let mut file = std::fs::File::open("world/entities/r.-1.-1.mca").unwrap();

        let mut bytes = vec![];

        let mut string = String::new();
        file.read_to_end(&mut bytes).unwrap();

        dbg!(bytes.len());
    }
}

mod tests {

    use std::io::Read;

    use crate::world::chunk::WorldChunk;


    #[test]
    pub fn load_100_chunks() {
        //im doing it with tests as bench is not implemented yet

        let mut vector = vec![];
        for i in 0..100 {
            let chunk = WorldChunk::load_chunk("../world/region", 0, 2);
            vector.push(chunk)
        }
        let mut length = 0;
        for chunk in vector {
            for block in chunk.blocks {
                length = length + 1;
            }
        }
        dbg!(length);
    }

    #[test]
    pub fn load_1000_chunks() {
        //im doing it with tests as bench is not implemented yet
        let mut vector = vec![];
        for i in 0..1000 {
            let chunk = WorldChunk::load_chunk("../world/region", 0, 2);
            vector.push(chunk)
        }
        println!("test");
    }
}
