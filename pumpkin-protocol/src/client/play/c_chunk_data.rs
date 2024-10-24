use std::collections::HashMap;

use crate::{bytebuf::ByteBuffer, BitSet, ClientPacket, VarInt};
use itertools::Itertools;
use pumpkin_macros::client_packet;
use pumpkin_world::{
    chunk::{ChunkData, SUBCHUNK_VOLUME},
    lighting::ChunkLighting,
    lighting2::ChunkLightData,
    DIRECT_PALETTE_BITS,
};

use super::ClientboundPlayPackets;

#[client_packet(ClientboundPlayPackets::ChunkData as i32)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl<'a> ClientPacket for CChunkData<'a> {
    fn write(&self, buf: &mut crate::bytebuf::ByteBuffer) {
        // Chunk X
        buf.put_i32(self.0.position.x);
        // Chunk Z
        buf.put_i32(self.0.position.z);

        let heightmap_nbt =
            fastnbt::to_bytes_with_opts(&self.0.blocks.heightmap, fastnbt::SerOpts::network_nbt())
                .unwrap();
        // Heightmaps
        buf.put_slice(&heightmap_nbt);

        let mut data_buf = ByteBuffer::empty();
        self.0.blocks.iter_subchunks().for_each(|chunk| {
            let block_count = chunk.iter().filter(|block| !block.is_air()).count() as i16;
            // Block count
            data_buf.put_i16(block_count);
            //// Block states

            let palette = chunk.iter().dedup().collect_vec();
            // TODO: make dynamic block_size work
            // TODO: make direct block_size work
            enum PaletteType {
                Indirect(u32),
                Direct,
            }
            let palette_type = {
                let palette_bit_len = 64 - (palette.len() as i64 - 1).leading_zeros();
                if palette_bit_len > 8 {
                    PaletteType::Direct
                } else if palette_bit_len > 3 {
                    PaletteType::Indirect(palette_bit_len)
                } else {
                    PaletteType::Indirect(4)
                }
                // TODO: fix indirect palette to work correctly
                // PaletteType::Direct
            };

            let mut block_data_array = Vec::new();
            match palette_type {
                PaletteType::Indirect(block_size) => {
                    // Bits per entry
                    data_buf.put_u8(block_size as u8);
                    // Palette length
                    data_buf.put_var_int(&VarInt(palette.len() as i32));
                    let mut palette_map = HashMap::new();
                    palette.iter().enumerate().for_each(|(i, id)| {
                        palette_map.insert(*id, i);
                        // Palette
                        data_buf.put_var_int(&VarInt(id.get_id_mojang_repr()));
                    });
                    for block_clump in chunk.chunks(64 / block_size as usize) {
                        let mut out_long: i64 = 0;
                        for block in block_clump.iter().rev() {
                            let index = palette_map
                                .get(block)
                                .expect("Its just got added, ofc it should be there");
                            out_long = out_long << block_size | (*index as i64);
                        }
                        block_data_array.push(out_long);
                    }
                }
                PaletteType::Direct => {
                    // Bits per entry
                    data_buf.put_u8(DIRECT_PALETTE_BITS as u8);
                    for block_clump in chunk.chunks(64 / DIRECT_PALETTE_BITS as usize) {
                        let mut out_long: i64 = 0;
                        let mut shift = 0;
                        for block in block_clump {
                            out_long |= (block.get_id() as i64) << shift;
                            shift += DIRECT_PALETTE_BITS;
                        }
                        block_data_array.push(out_long);
                    }
                }
            }

            // Data array length
            // TODO: precompute this and omit making the `block_data_array`
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
        buf.put_slice(data_buf.buf());

        // TODO: block entities
        buf.put_var_int(&VarInt(0));

        let chunk_light_data = ChunkLightData::initialize(
            pumpkin_world::lighting2::ChunkCoordinates {
                x: self.0.position.x,
                z: self.0.position.z,
            },
            &self.0.blocks,
        );

        let (sky_light_mask, empty_sky_light_mask, subchunks) = chunk_light_data.packet_data();

        // Sky Light Mask
        // All of the chunks, this is not optimal and uses way more data than needed but will be
        // overhauled with full lighting system.
        buf.put_bit_set(&BitSet(VarInt(1), &[sky_light_mask]));
        // Block Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), &[0]));
        // Empty Sky Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), &[empty_sky_light_mask]));
        // Empty Block Light Mask
        buf.put_bit_set(&BitSet(VarInt(1), &[0]));

        let debug = self.0.position.x == -1 && self.0.position.z == 0;

        if debug {
            println!("{:b}", sky_light_mask);
            println!("{:b}", empty_sky_light_mask);
            println!("New Chunk? {} ", subchunks.len());
        }
        buf.put_var_int(&subchunks.len().into());
        for (i, subchunk) in subchunks.into_iter().enumerate() {
            buf.put_var_int(&VarInt(subchunk.len() as i32));
            buf.put_slice(&subchunk);
            // if debug {
            //     print!("Start Subchunk: ");
            //     subchunk.iter().for_each(|v| print!("{:X}", v));
            //     print!("\n");
            // }
        }

        // Block Lighting
        buf.put_var_int(&VarInt(0));
    }
}
