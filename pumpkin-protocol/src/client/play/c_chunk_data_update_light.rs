use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    BitSet, ClientPacket, VarInt,
};

pub struct CChunkDataUpdateLight {
    chunk_x: i32,
    chunk_y: i32,
    heightmaps: Vec<u8>,
    data: Vec<u8>,
    block_entites: Vec<BlockEntity>,
    sky_light_mask: BitSet,
    block_light_mask: BitSet,
    empty_sky_light_mask: BitSet,
    sky_lights: Vec<SkyLight>,
    block_lights: Vec<BlockLight>,
}

impl CChunkDataUpdateLight {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chunk_x: i32,
        chunk_y: i32,
        heightmaps: Vec<u8>,
        data: Vec<u8>,
        block_entites: Vec<BlockEntity>,
        sky_light_mask: BitSet,
        block_light_mask: BitSet,
        empty_sky_light_mask: BitSet,
        sky_lights: Vec<SkyLight>,
        block_lights: Vec<BlockLight>,
    ) -> Self {
        Self {
            chunk_x,
            chunk_y,
            heightmaps,
            data,
            block_entites,
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            sky_lights,
            block_lights,
        }
    }
}

impl Packet for CChunkDataUpdateLight {
    const PACKET_ID: VarInt = 0x27;
}

impl ClientPacket for CChunkDataUpdateLight {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i32(self.chunk_x);
        bytebuf.put_i32(self.chunk_y);
        bytebuf.put_slice(&self.heightmaps);
        bytebuf.put_var_int(self.data.len() as VarInt);
        bytebuf.put_slice(&self.data);
        bytebuf.put_list::<BlockEntity>(&self.block_entites, |p, v| {
            p.put_u8(v.packed_xz);
            p.put_i16(v.y);
            p.put_var_int(v.typee);
            p.put_slice(&v.data);
        });
        bytebuf.put_bit_set(&self.sky_light_mask);
        bytebuf.put_bit_set(&self.block_light_mask);
        bytebuf.put_bit_set(&self.empty_sky_light_mask);
        bytebuf.put_list::<SkyLight>(&self.sky_lights, |p, v| {
            p.put_var_int(v.array.len() as VarInt);
            p.put_slice(&v.array);
        });
        bytebuf.put_list::<BlockLight>(&self.block_lights, |p, v| {
            p.put_var_int(v.array.len() as VarInt);
            p.put_slice(&v.array);
        });
    }
}

pub struct BlockEntity {
    packed_xz: u8,
    y: i16,
    typee: VarInt,
    data: Vec<u8>,
}

pub struct SkyLight {
    pub array: Vec<u8>,
}

pub struct BlockLight {
    pub array: Vec<u8>,
}
