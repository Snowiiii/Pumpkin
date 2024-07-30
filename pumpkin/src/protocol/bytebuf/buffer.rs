use crate::protocol::{VarInt, VarLong};

use super::{Endian, CONTINUE_BIT, SEGMENT_BITS};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::{
    fmt::Debug,
    io::{Error, ErrorKind, Read, Result, Write},
};

/// A byte buffer object specifically turned to easily read and write binary values
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ByteBuffer {
    data: Vec<u8>,
    wpos: usize,
    rpos: usize,
    wbit: usize,
    rbit: usize,
    endian: Endian,
}

impl From<&[u8]> for ByteBuffer {
    fn from(val: &[u8]) -> Self {
        ByteBuffer::from_bytes(val)
    }
}

impl From<Vec<u8>> for ByteBuffer {
    fn from(val: Vec<u8>) -> Self {
        ByteBuffer::from_vec(val)
    }
}

impl From<ByteBuffer> for Vec<u8> {
    fn from(val: ByteBuffer) -> Self {
        val.into_vec()
    }
}

impl Default for ByteBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Read for ByteBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.flush_bits();
        let read_len = std::cmp::min(self.data.len() - self.rpos, buf.len());
        let range = self.rpos..self.rpos + read_len;
        for (i, val) in self.data[range].iter().enumerate() {
            buf[i] = *val;
        }
        self.rpos += read_len;
        Ok(read_len)
    }
}

impl Write for ByteBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Debug for ByteBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let rpos = if self.rbit > 0 {
            self.rpos + 1
        } else {
            self.rpos
        };

        let read_len = self.data.len() - rpos;
        let mut remaining_data = vec![0; read_len];
        let range = rpos..rpos + read_len;
        for (i, val) in self.data[range].iter().enumerate() {
            remaining_data[i] = *val;
        }

        write!(
            f,
            "ByteBuffer {{ remaining_data: {:?}, total_data: {:?}, wpos: {:?}, rpos: {:?}, endian: {:?} }}",
            remaining_data, self.data, self.wpos, self.rpos, self.endian
        )
    }
}

macro_rules! read_number {
    ($self:ident, $name:ident, $offset:expr) => {{
        $self.flush_bits();
        if $self.rpos + $offset > $self.data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "could not read enough bits from buffer",
            ));
        }
        let range = $self.rpos..$self.rpos + $offset;
        $self.rpos += $offset;

        Ok(match $self.endian {
            Endian::BigEndian => BigEndian::$name(&$self.data[range]),
            Endian::LittleEndian => LittleEndian::$name(&$self.data[range]),
        })
    }};
}

impl ByteBuffer {
    /// Construct a new, empty, ByteBuffer
    pub fn new() -> ByteBuffer {
        ByteBuffer {
            data: vec![],
            wpos: 0,
            rpos: 0,
            rbit: 0,
            wbit: 0,
            endian: Endian::BigEndian,
        }
    }

    /// Construct a new ByteBuffer filled with the data array.
    pub fn from_bytes(bytes: &[u8]) -> ByteBuffer {
        let mut buffer = ByteBuffer::new();
        buffer.write_bytes(bytes);
        buffer
    }

    /// Constructs a new ByteBuffer from an existing vector. This
    /// function takes ownership of the vector
    pub fn from_vec(vec: Vec<u8>) -> ByteBuffer {
        let len = vec.len();
        ByteBuffer {
            data: vec,
            wpos: len,
            rpos: 0,
            rbit: 0,
            wbit: 0,
            endian: Endian::BigEndian,
        }
    }

    /// Return the buffer size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the buffer and reinitialize the reading and writing cursors
    pub fn clear(&mut self) {
        self.data.clear();
        self.reset_cursors();
        self.reset_bits_cursors();
    }

    /// Reinitialize the reading and writing cursor
    pub fn reset_cursors(&mut self) {
        self.wpos = 0;
        self.rpos = 0;
    }

    /// Reinitialize the bit reading and bit writing cursor
    pub fn reset_bits_cursors(&mut self) {
        self.rbit = 0;
        self.wbit = 0;
    }

    /// Change the buffer size to size.
    ///
    /// _Note_: You cannot shrink a buffer with this method
    pub fn resize(&mut self, size: usize) {
        let diff = size - self.data.len();
        if diff > 0 {
            self.data.extend(std::iter::repeat(0).take(diff))
        }
    }

    /// Set the byte order of the buffer
    ///
    /// _Note_: By default the buffer uses big endian order
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    /// Returns the current byte order of the buffer
    pub fn endian(&self) -> Endian {
        self.endian
    }

    // Write operations

    /// Append a byte array to the buffer. The buffer is automatically extended if needed
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// # use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_bytes(&vec![0x1, 0xFF, 0x45]); // buffer contains [0x1, 0xFF, 0x45]
    /// ```
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.flush_bits();

        let size = bytes.len() + self.wpos;

        if size > self.data.len() {
            self.resize(size);
        }

        for v in bytes {
            self.data[self.wpos] = *v;
            self.wpos += 1;
        }
    }

    pub fn write_bool(&mut self, v: bool) {
        if v {
            self.write_u8(1);
        } else {
            self.write_u8(0);
        }
    }

    pub fn write_bytes_len(&mut self, bytes: &[u8], max_len: usize) {
        self.flush_bits();

        let size = bytes.len() + self.wpos;
        if size > max_len {
            eprintln!("Write: size > max size");
            return;
        }

        if size > self.data.len() {
            self.resize(size);
        }

        for v in bytes {
            self.data[self.wpos] = *v;
            self.wpos += 1;
        }
    }

    /// Append a byte (8 bits value) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_u8(1) // buffer contains [0x1]
    /// ```
    pub fn write_u8(&mut self, val: u8) {
        self.write_bytes(&[val]);
    }

    /// Same as `write_u8()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i8(&mut self, val: i8) {
        self.write_u8(val as u8);
    }

    /// Append a word (16 bits value) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_u16(1) // buffer contains [0x00, 0x1] if little endian
    /// ```
    pub fn write_u16(&mut self, val: u16) {
        let mut buf = [0; 2];

        match self.endian {
            Endian::BigEndian => BigEndian::write_u16(&mut buf, val),
            Endian::LittleEndian => LittleEndian::write_u16(&mut buf, val),
        };

        self.write_bytes(&buf);
    }

    /// Same as `write_u16()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i16(&mut self, val: i16) {
        self.write_u16(val as u16);
    }

    /// Append a double word (32 bits value) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_u32(1) // buffer contains [0x00, 0x00, 0x00, 0x1] if little endian
    /// ```
    pub fn write_u32(&mut self, val: u32) {
        let mut buf = [0; 4];

        match self.endian {
            Endian::BigEndian => BigEndian::write_u32(&mut buf, val),
            Endian::LittleEndian => LittleEndian::write_u32(&mut buf, val),
        };

        self.write_bytes(&buf);
    }

    /// Same as `write_u32()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i32(&mut self, val: i32) {
        self.write_u32(val as u32);
    }

    pub fn write_var_int(&mut self, value: VarInt) {
        let mut val = value as u32;
        for _ in 0..5 {
            let mut b: u8 = val as u8 & 0b01111111;
            val >>= 7;
            if val != 0 {
                b |= 0b10000000;
            }
            self.write_u8(b);
            if val == 0 {
                break;
            }
        }
    }

    /// Append a quaddruple word (64 bits value) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_u64(1) // buffer contains [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1] if little endian
    /// ```
    pub fn write_u64(&mut self, val: u64) {
        let mut buf = [0; 8];
        match self.endian {
            Endian::BigEndian => BigEndian::write_u64(&mut buf, val),
            Endian::LittleEndian => LittleEndian::write_u64(&mut buf, val),
        };

        self.write_bytes(&buf);
    }

    /// Same as `write_u64()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i64(&mut self, val: i64) {
        self.write_u64(val as u64);
    }

    /// Append a 32 bits floating point number to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_f32(0.1)
    /// ```
    pub fn write_f32(&mut self, val: f32) {
        let mut buf = [0; 4];

        match self.endian {
            Endian::BigEndian => BigEndian::write_f32(&mut buf, val),
            Endian::LittleEndian => LittleEndian::write_f32(&mut buf, val),
        };

        self.write_bytes(&buf);
    }

    /// Append a 64 bits floating point number to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_f64(0.1)
    /// ```
    pub fn write_f64(&mut self, val: f64) {
        let mut buf = [0; 8];

        match self.endian {
            Endian::BigEndian => BigEndian::write_f64(&mut buf, val),
            Endian::LittleEndian => LittleEndian::write_f64(&mut buf, val),
        };
        self.write_bytes(&buf);
    }

    /// Append a string to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// *Format* The format is `(u32)size + size * (u8)characters`
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_string("Hello")
    /// ```
    pub fn write_string(&mut self, val: &str) {
        self.write_string_len(val, 32767)
    }

    pub fn write_string_len(&mut self, val: &str, max_len: usize) {
        self.write_var_int(val.len() as VarInt);
        self.write_bytes_len(val.as_bytes(), max_len);
    }

    pub fn write_string_array(&mut self, array: &[String]) {
        for string in array {
            self.write_string(string)
        }
    }

    // Read operations

    /// Read a defined amount of raw bytes, or return an IO error if not enough bytes are
    /// available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
        self.flush_bits();
        if self.rpos + size > self.data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "could not read enough bytes from buffer",
            ));
        }
        let range = self.rpos..self.rpos + size;
        let mut res = Vec::<u8>::new();
        res.write_all(&self.data[range])?;
        self.rpos += size;
        Ok(res)
    }

    /// Read one byte, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![0x1]);
    /// let value = buffer.read_u8().unwrap(); //Value contains 1
    /// ```
    pub fn read_u8(&mut self) -> Result<u8> {
        self.flush_bits();
        if self.rpos >= self.data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "could not read enough bits from buffer",
            ));
        }
        let pos = self.rpos;
        self.rpos += 1;
        Ok(self.data[pos])
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    /// Same as `read_u8()` but for signed values
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read a 2-bytes long value, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![0x0, 0x1]);
    /// let value = buffer.read_u16().unwrap(); //Value contains 1
    /// ```
    pub fn read_u16(&mut self) -> Result<u16> {
        read_number!(self, read_u16, 2)
    }

    /// Same as `read_u16()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    /// Read a four-bytes long value, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x1]);
    /// let value = buffer.read_u32().unwrap(); // Value contains 1
    /// ```
    pub fn read_u32(&mut self) -> Result<u32> {
        read_number!(self, read_u32, 4)
    }

    /// Same as `read_u32()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    /// Read an eight bytes long value, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1]);
    /// let value = buffer.read_u64().unwrap(); //Value contains 1
    /// ```
    pub fn read_u64(&mut self) -> Result<u64> {
        read_number!(self, read_u64, 8)
    }

    /// Reads a boolean. If true, the closure is called, and the returned value is
    /// wrapped in Some. Otherwise, this returns None.
    pub fn read_option<T>(&mut self, val: impl FnOnce(&mut ByteBuffer) -> T) -> Result<Option<T>> {
        if self.read_bool()? {
            Ok(Some(val(self)))
        } else {
            Ok(None)
        }
    }
    /// Writes `true` if the option is Some, or `false` if None. If the option is
    /// some, then it also calls the `write` closure.
    pub fn write_option<T>(&mut self, val: &Option<T>, write: impl FnOnce(&mut ByteBuffer, &T)) {
        self.write_bool(val.is_some());
        match val {
            Some(v) => write(self, v),
            None => {}
        }
    }

    pub fn read_var_int(&mut self) -> Result<VarInt> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        loop {
            let read = self.read_u8()?;

            value |= ((read & SEGMENT_BITS) as i32) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err(Error::new(ErrorKind::InvalidData, "VarInt is too big"));
            }
        }

        Ok(value)
    }

    pub fn read_var_long(&mut self) -> Result<VarLong> {
        let mut value: i64 = 0;
        let mut position: i64 = 0;

        loop {
            let read = self.read_u8()?;

            value |= ((read & SEGMENT_BITS) as i64) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 64 {
                return Err(Error::new(ErrorKind::InvalidData, "VarLong is too big"));
            }
        }

        Ok(value)
    }

    /// Same as `read_u64()` but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(self.read_u64()? as i64)
    }

    /// Read a 32 bits floating point value, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_f32(&mut self) -> Result<f32> {
        read_number!(self, read_f32, 4)
    }

    /// Read a 64 bits floating point value, or return an IO error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_f64(&mut self) -> Result<f64> {
        read_number!(self, read_f64, 8)
    }

    /// Read a string.
    ///
    /// _Note_: First it reads a 32 bits value representing the size, then 'size' raw bytes
    ///         that  must be encoded as UTF8.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_string(&mut self) -> Result<String> {
        self.read_string_len(32767)
    }

    pub fn read_string_len(&mut self, max_size: usize) -> Result<String> {
        let size = self.read_var_int()?;
        if size as usize > max_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "String length is bigger than max size",
            ));
        }
        let data = self.read_bytes(size as usize)?;
        if data.len() > max_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "String is bigger than max size",
            ));
        }
        match String::from_utf8(data) {
            Ok(string_result) => Ok(string_result),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    /// Reads 16 bytes from the buffer, and returns that as a big endian UUID.
    pub fn read_uuid(&mut self) -> Result<uuid::Uuid> {
        let mut bytes = [0u8; 16];
        self.read_exact(&mut bytes)?;
        uuid::Uuid::from_slice(&bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// This writes a UUID into the buffer (in big endian format).
    pub fn write_uuid(&mut self, v: uuid::Uuid) {
        self.write_bytes(v.as_bytes());
    }

    // Other

    /// Dump the byte buffer to a string.
    pub fn to_hex_dump(&self) -> String {
        let mut str = String::new();
        for b in &self.data {
            str = str + &format!("0x{:01$x} ", b, 2);
        }
        str.pop();
        str
    }

    /// Return the position of the reading cursor
    pub fn get_rpos(&self) -> usize {
        self.rpos
    }

    /// Set the reading cursor position.
    /// _Note_: Sets the reading cursor to `min(newPosition, self.len())` to prevent overflow
    pub fn set_rpos(&mut self, rpos: usize) {
        self.rpos = std::cmp::min(rpos, self.data.len());
    }

    /// Return the writing cursor position
    pub fn get_wpos(&self) -> usize {
        self.wpos
    }

    /// Set the writing cursor position.
    /// _Note_: Sets the writing cursor to `min(newPosition, self.len())` to prevent overflow
    pub fn set_wpos(&mut self, wpos: usize) {
        self.wpos = std::cmp::min(wpos, self.data.len());
    }

    /// Return the raw byte buffer bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Return the raw byte buffer as a Vec<u8>.
    #[deprecated(
        since = "2.1.0",
        note = "use `as_bytes().to_vec()` or `into_vec()` instead"
    )]
    pub fn into_bytes(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    /// Return the raw byte buffer as a Vec<u8>.
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    //Bit manipulation functions

    /// Read 1 bit. Return true if the bit is set to 1, otherwhise, return false.
    ///
    /// _Note_: Bits are read from left to right
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![128]); // 10000000b
    /// let value1 = buffer.read_bit().unwrap(); //value1 contains true (eg: bit is 1)
    /// let value2 = buffer.read_bit().unwrap(); //value2 contains false (eg: bit is 0)
    /// ```
    pub fn read_bit(&mut self) -> Result<bool> {
        if self.rpos >= self.data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "could not read enough bits from buffer",
            ));
        }
        let bit = self.data[self.rpos] & (1 << (7 - self.rbit)) != 0;
        self.rbit += 1;
        if self.rbit > 7 {
            self.flush_rbits();
        }
        Ok(bit)
    }

    /// Read n bits. an return the corresponding value an u64.
    ///
    /// _Note_: We cannot read more than 64 bits
    ///
    /// _Note_: Bits are read from left to right
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::from_bytes(&vec![128]); // 10000000b
    /// let value = buffer.read_bits(3).unwrap(); // value contains 4 (eg: 100b)
    /// ```
    pub fn read_bits(&mut self, n: u8) -> Result<u64> {
        if n > 64 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "cannot read more than 64 bits",
            ));
        }

        if n == 0 {
            Ok(0)
        } else {
            Ok((u64::from(self.read_bit()?) << (n - 1)) | self.read_bits(n - 1)?)
        }
    }

    /// Discard all the pending bits available for reading or writing and place the corresponding cursor to the next byte.
    ///
    /// _Note_: If no bits are currently read or written, this function does nothing.
    ///
    /// #Example
    ///
    /// ```text
    /// 10010010 | 00000001
    /// ^
    /// 10010010 | 00000001 // read_bit called
    ///  ^
    /// 10010010 | 00000001 // flush_bit() called
    ///            ^
    /// ```
    pub fn flush_bits(&mut self) {
        if self.rbit > 0 {
            self.flush_rbits();
        }
        if self.wbit > 0 {
            self.flush_wbits();
        }
    }

    fn flush_rbits(&mut self) {
        self.rpos += 1;
        self.rbit = 0
    }

    fn flush_wbits(&mut self) {
        self.wpos += 1;
        self.wbit = 0
    }

    /// Append 1 bit value to the buffer.
    /// The bit is appended like this :
    ///
    /// ```text
    /// ...| XXXXXXXX | 10000000 |....
    /// ```
    pub fn write_bit(&mut self, bit: bool) {
        let size = self.wpos + 1;
        if size > self.data.len() {
            self.resize(size);
        }

        if bit {
            self.data[self.wpos] |= 1 << (7 - self.wbit);
        }

        self.wbit += 1;

        if self.wbit > 7 {
            self.wbit = 0;
            self.wpos += 1;
        }
    }

    /// Write the given value as a sequence of n bits
    ///
    /// #Example
    ///
    /// ```
    /// #  use bytebuffer::*;
    /// let mut buffer = ByteBuffer::new();
    /// buffer.write_bits(4, 3); // append 100b
    /// ```
    pub fn write_bits(&mut self, value: u64, n: u8) {
        if n > 0 {
            self.write_bit((value >> (n - 1)) & 1 != 0);
            self.write_bits(value, n - 1);
        }
    }
}
