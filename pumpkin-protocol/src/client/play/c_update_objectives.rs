use bytes::{BufMut, BytesMut};
use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, NumberFormat, VarInt};

#[client_packet("play:set_objective")]
pub struct CUpdateObjectives<'a> {
    objective_name: &'a str,
    mode: u8,
    display_name: TextComponent<'a>,
    render_type: VarInt,
    number_format: Option<NumberFormat<'a>>,
}

impl<'a> CUpdateObjectives<'a> {
    pub fn new(
        objective_name: &'a str,
        mode: Mode,
        display_name: TextComponent<'a>,
        render_type: RenderType,
        number_format: Option<NumberFormat<'a>>,
    ) -> Self {
        Self {
            objective_name,
            mode: mode as u8,
            display_name,
            render_type: VarInt(render_type as i32),
            number_format,
        }
    }
}

impl ClientPacket for CUpdateObjectives<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_string(self.objective_name);
        bytebuf.put_u8(self.mode);
        if self.mode == 0 || self.mode == 2 {
            bytebuf.put_slice(&self.display_name.encode());
            bytebuf.put_var_int(&self.render_type);
            bytebuf.put_option(&self.number_format, |p, v| {
                match v {
                    NumberFormat::Blank => {
                        p.put_var_int(&VarInt(0));
                    }
                    NumberFormat::Styled(style) => {
                        p.put_var_int(&VarInt(1));
                        // TODO
                        p.put_slice(&pumpkin_nbt::serializer::to_bytes_unnamed(style).unwrap());
                    }
                    NumberFormat::Fixed(text_component) => {
                        p.put_var_int(&VarInt(2));
                        p.put_slice(&text_component.encode());
                    }
                }
            });
        }
    }
}

#[repr(u8)]
pub enum Mode {
    Add,
    Remove,
    Update,
}

#[repr(i32)]
pub enum RenderType {
    Integer,
    Hearts,
}
