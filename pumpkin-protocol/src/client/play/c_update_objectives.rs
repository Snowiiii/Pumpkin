use pumpkin_core::text::{style::Style, TextComponent};
use pumpkin_macros::packet;

use crate::{ClientPacket, VarInt};

#[packet(0x5E)]
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
        mode: u8,
        display_name: TextComponent<'a>,
        render_type: VarInt,
        number_format: Option<NumberFormat<'a>>,
    ) -> Self {
        Self {
            objective_name,
            mode,
            display_name,
            render_type,
            number_format,
        }
    }
}

impl<'a> ClientPacket for CUpdateObjectives<'a> {
    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
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
                        p.put_slice(&fastnbt::to_bytes(style).unwrap());
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

pub enum NumberFormat<'a> {
    /// Show nothing
    Blank,
    /// The styling to be used when formatting the score number
    Styled(Style<'a>),
    /// The text to be used as placeholder.
    Fixed(TextComponent<'a>),
}
