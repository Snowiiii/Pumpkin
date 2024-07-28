use super::Endian;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::{
    fmt::Debug,
    io::{Error, ErrorKind, Read, Result, Write},
};

/// A byte buffer object specifically turned to easily read and write binary values
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    rpos: usize,
    rbit: usize,
    endian: Endian,
}

impl<'a> From<&'a [u8]> for ByteReader<'a> {
    fn from(val: &'a [u8]) -> Self {
        ByteReader::from_bytes(val)
    }
}

impl<'a> Read for ByteReader<'a> {
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

impl<'a> Debug for ByteReader<'a> {
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
            "ByteReader {{ remaining_data: {:?}, total_data: {:?}, rpos: {:?}, endian: {:?} }}",
            remaining_data, self.data, self.rpos, self.endian
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

impl<'a> ByteReader<'a> {
    /// Construct a new ByteReader filled with the data array.
    pub fn from_bytes(bytes: &[u8]) -> ByteReader {
        ByteReader {
            data: bytes,
            rpos: 0,
            rbit: 0,
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

    /// Reinitialize the reading cursor
    pub fn reset_cursors(&mut self) {
        self.rpos = 0;
    }

    /// Reinitialize the bit reading cursor
    pub fn reset_bits_cursors(&mut self) {
        self.rbit = 0;
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
    /// let data = vec![0x1];
    /// let mut buffer = ByteReader::from_bytes(&data);
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
    /// let data = vec![0x0, 0x1];
    /// let mut buffer = ByteReader::from_bytes(&data);
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
    /// let data = vec![0x0, 0x0, 0x0, 0x1];
    /// let mut buffer = ByteReader::from_bytes(&data);
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
    /// let data = vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];
    /// let mut buffer = ByteReader::from_bytes(&data);
    /// let value = buffer.read_u64().unwrap(); //Value contains 1
    /// ```
    pub fn read_u64(&mut self) -> Result<u64> {
        read_number!(self, read_u64, 8)
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
        let size = self.read_u32()?;
        match String::from_utf8(self.read_bytes(size as usize)?) {
            Ok(string_result) => Ok(string_result),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    // Other

    /// Dump the byte buffer to a string.
    pub fn to_hex_dump(&self) -> String {
        let mut str = String::new();
        for b in self.data {
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

    /// Return the raw byte buffer bytes.
    pub fn as_bytes(&self) -> &[u8] {
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
    /// let data = vec![128];
    /// let mut buffer = ByteReader::from_bytes(&data); // 10000000b
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
    /// let data = vec![128];
    /// let mut buffer = ByteReader::from_bytes(&data); // 10000000b
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

    /// Discard all the pending bits available for reading and place the corresponding cursor to the next byte.
    ///
    /// _Note_: If no bits are currently read, this function does nothing.
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
    }

    fn flush_rbits(&mut self) {
        self.rpos += 1;
        self.rbit = 0
    }
}
