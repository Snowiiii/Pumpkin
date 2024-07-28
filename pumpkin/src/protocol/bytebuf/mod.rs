pub mod buffer;
pub mod reader;

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;
/// An enum to represent the byte order of the ByteBuffer object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endian {
    BigEndian,
    LittleEndian,
}
