use std::collections::HashMap;

use crate::{bytebuf::ByteBuffer, BitSet, ClientPacket, VarInt};
use fastnbt::LongArray;
use itertools::Itertools;
use pumpkin_macros::packet;
use pumpkin_world::chunk::ChunkData;

#[packet(0x27)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl<'a> ClientPacket for CChunkData<'a> {
    fn write(&self, buf: &mut crate::bytebuf::ByteBuffer) {
        // Chunk X
        buf.put_i32(self.0.position.0);
        // Chunk Z
        buf.put_i32(self.0.position.1);

        let heightmap_nbt =
            fastnbt::to_bytes_with_opts(&self.0.heightmaps, fastnbt::SerOpts::network_nbt())
                .unwrap();
        // Heightmaps
        buf.put_slice(&heightmap_nbt);

        let mut data_buf = ByteBuffer::empty();
        self.0
            .blocks
            .iter()
            .chunks(16 * 16 * 16)
            .into_iter()
            .for_each(|chunk| {
                let chunk = chunk.collect_vec();
                let block_count = chunk
                    .iter()
                    .filter(|block| ***block != 0 && ***block != 12959 && ***block != 12958)
                    .count() as i16;
                // Block count
                data_buf.put_i16(block_count);
                //// Block states
                let palette = chunk.clone().into_iter().dedup().collect_vec();
                let mut palette_map = HashMap::new();
                let block_size = 8;
                // let block_size = {
                //     let palette_bit_len = palette.len().leading_zeros();
                //     if palette_bit_len > 3 {
                //         palette_bit_len
                //     } else {
                //         4
                //     }
                // };
                // Bits per entry
                data_buf.put_u8(block_size as u8);
                // Palette length
                data_buf.put_var_int(&VarInt(palette.len() as i32));
                palette.iter().enumerate().for_each(|(i, id)| {
                    palette_map.insert(*id, i);
                    // Palette
                    data_buf.put_var_int(&VarInt(**id as i32));
                });
                let mut block_data_array = Vec::new();
                for block_clump in chunk.chunks(64 / block_size as usize) {
                    let mut out_long: i64 = 0;
                    let mut first = true;
                    for block in block_clump {
                        if first {
                            first = false;
                        } else {
                            out_long = out_long << block_size;
                        }
                        let index = palette_map
                            .get(block)
                            .expect("Its just got added, ofc it should be there");
                        out_long = out_long | *index as i64;
                    }
                    block_data_array.push(out_long);
                }

                // Data array length
                data_buf.put_var_int(&VarInt(block_data_array.len() as i32));
                // Data array
                for data_int in block_data_array {
                    data_buf.put_i64(data_int);
                }

                //// Biomes
                // TODO: make biomes work
                data_buf.put_u8(0);
                data_buf.put_var_int(&VarInt(0));
                data_buf.put_var_int(&VarInt(0));
            });

        // Size
        buf.put_var_int(&VarInt(data_buf.buf().len() as i32));
        // Data
        buf.put_slice(&data_buf.buf());

        // TODO: block entities
        buf.put_var_int(&VarInt(0));

        // TODO
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));
        buf.put_bit_set(&BitSet(VarInt(1), vec![0]));
        // buf.put_bit_set(&BitSet(VarInt(0), vec![]));
        // buf.put_bit_set(&BitSet(VarInt(0), vec![]));
        // buf.put_bit_set(&BitSet(VarInt(0), vec![]));
        // buf.put_bit_set(&BitSet(VarInt(0), vec![]));

        buf.put_var_int(&VarInt(0));
        buf.put_var_int(&VarInt(0));
    }
}
