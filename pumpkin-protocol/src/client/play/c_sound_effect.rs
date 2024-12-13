use bytes::{BufMut, BytesMut};
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, IDOrSoundEvent, SoundCategory, SoundEvent, VarInt};

#[client_packet("play:sound")]
pub struct CSoundEffect {
    sound_event: IDOrSoundEvent,
    sound_category: VarInt,
    effect_position_x: i32,
    effect_position_y: i32,
    effect_position_z: i32,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CSoundEffect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sound_id: VarInt,
        sound_event: Option<SoundEvent>,
        sound_category: SoundCategory,
        effect_position_x: f64,
        effect_position_y: f64,
        effect_position_z: f64,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event: IDOrSoundEvent {
                id: VarInt(sound_id.0 + 1),
                sound_event,
            },
            sound_category: VarInt(sound_category as i32),
            effect_position_x: (effect_position_x * 8.0) as i32,
            effect_position_y: (effect_position_y * 8.0) as i32,
            effect_position_z: (effect_position_z * 8.0) as i32,
            volume,
            pitch,
            seed,
        }
    }
}

impl ClientPacket for CSoundEffect {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(&self.sound_event.id);
        if self.sound_event.id.0 == 0 {
            if let Some(test) = &self.sound_event.sound_event {
                bytebuf.put_string(&test.sound_name);

                bytebuf.put_option(&test.range, |p, v| {
                    p.put_f32(*v);
                });
            }
        }
        bytebuf.put_var_int(&self.sound_category);
        bytebuf.put_i32(self.effect_position_x);
        bytebuf.put_i32(self.effect_position_y);
        bytebuf.put_i32(self.effect_position_z);
        bytebuf.put_f32(self.volume);
        bytebuf.put_f32(self.pitch);
        bytebuf.put_f64(self.seed);
    }
}
