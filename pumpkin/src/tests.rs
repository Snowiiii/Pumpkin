#![feature(test)]

#[allow(soft_unstable)]
mod tests {
    use std::{env, time::SystemTime};

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
