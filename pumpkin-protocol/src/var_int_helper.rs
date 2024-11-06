use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum VarIntDecodeError {
    #[error("incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}

pub(super) const SEGMENT_BITS: u8 = 0b01111111;
pub(super) const CONTINUE_BIT: u8 = 0b10000000;

#[inline]
pub(super) fn read_byte_slice(buff: &mut &[u8]) -> Option<u8> {
    buff.split_first().map(|(&byte, rest)| {
        *buff = rest;
        byte
    })
}

pub trait VarEncodedInteger:
    Debug + Copy + Clone + PartialEq + Eq + Ord + PartialOrd + Hash
{
    /// The maximum number of bytes this `VarEncodedInteger` could occupy when read from and
    /// written to the Minecraft protocol.
    const MAX_SIZE: usize;
    type IntType: From<Self> + Into<Self>;

    /// Returns the exact number of bytes this varint will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    fn written_size(self) -> usize;

    fn decode(read: impl FnMut() -> Option<u8>) -> Result<Self, VarIntDecodeError>;

    fn encode<T>(self, writer: impl FnOnce(&[u8]) -> T) -> T;

    fn decode_from_slice(buff: &mut &[u8]) -> Result<Self, VarIntDecodeError> {
        Self::decode(|| read_byte_slice(buff))
    }

    fn try_decode<E>(
        mut reader: impl FnMut() -> Result<Option<u8>, E>,
        map_err: impl FnOnce(VarIntDecodeError) -> E,
    ) -> Result<Self, E> {
        let mut reader_error = None;
        let res = Self::decode(|| {
            reader().unwrap_or_else(|err| {
                reader_error = Some(err);
                None
            })
        });

        match reader_error {
            Some(err) => Err(err),
            None => res.map_err(map_err),
        }
    }

    fn read_from(mut reader: impl Read) -> io::Result<Self> {
        Self::try_decode(
            || {
                let mut byte = 0_u8;
                reader
                    .read(std::slice::from_mut(&mut byte))
                    .map(|len| match len {
                        1 => Some(byte),
                        0 => None,
                        _ => unreachable!(),
                    })
            },
            |err| match err {
                VarIntDecodeError::Incomplete => io::ErrorKind::UnexpectedEof.into(),
                VarIntDecodeError::TooLarge => io::ErrorKind::InvalidData.into(),
            },
        )
    }

    fn write_to(self, mut writer: impl Write) -> io::Result<()> {
        self.encode(move |var_int| writer.write_all(var_int))
    }
}

macro_rules! impl_var_int {
    ($var_int: ident ($inner_ty: ty) {
        max = $max: literal
        $ty: ty => $unsigned_ty: ty
    }) => {
        const _ASSERT_EQ_TYPES: fn($ty) -> $inner_ty = std::convert::identity;

        const _: () = const { assert!(<$ty>::BITS == <$unsigned_ty>::BITS) };

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $var_int($inner_ty);

        impl $var_int {
            pub const fn new(x: $inner_ty) -> Self {
                Self(x)
            }

            pub const fn get(self) -> $inner_ty {
                self.0
            }
        }

        impl VarEncodedInteger for $var_int {
            const MAX_SIZE: usize = $max;

            type IntType = $ty;

            fn written_size(self) -> usize {
                let len = match NonZero::new(self.0 as $unsigned_ty) {
                    None => 1,
                    Some(n) => ((n.ilog2() / 7) + 1) as usize,
                };

                debug_assert_eq!(self.encode(|buff| buff.len()), len);

                len
            }

            fn decode(mut read: impl FnMut() -> Option<u8>) -> Result<Self, VarIntDecodeError> {
                let mut val = 0;
                for i in 0..Self::MAX_SIZE {
                    let byte = read().ok_or(VarIntDecodeError::Incomplete)?;

                    val |= (Self::IntType::from(byte) & 0b01111111) << (i * 7);
                    if byte & 0b10000000 == 0 {
                        return Ok(Self(val));
                    }
                }
                Err(VarIntDecodeError::TooLarge)
            }

            fn encode<T>(self, writer: impl FnOnce(&[u8]) -> T) -> T {
                let mut i: u8 = 0;
                let mut scratch = [0_u8; Self::MAX_SIZE];
                let mut write = |byte: u8| {
                    scratch[i as usize] = byte;
                    i += 1;
                };
                let mut value = self.0 as $unsigned_ty;
                loop {
                    if (value & const { !(var_int_helper::SEGMENT_BITS as $unsigned_ty) }) == 0 {
                        let byte = value as u8;
                        write(byte);
                        break writer(&scratch[..i as usize]);
                    }

                    let byte = ((value as u8) & var_int_helper::SEGMENT_BITS)
                        | var_int_helper::CONTINUE_BIT;
                    write(byte);

                    value >>= 7;
                }
            }
        }

        impl From<u8> for $var_int {
            fn from(value: u8) -> Self {
                Self(<$ty>::from(value))
            }
        }

        impl From<$unsigned_ty> for $var_int {
            fn from(value: $unsigned_ty) -> Self {
                // Same number of bits this is a lossless transmute
                Self(value as $ty)
            }
        }

        impl TryFrom<usize> for $var_int {
            type Error = <$ty as TryFrom<usize>>::Error;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                <$ty>::try_from(value).map(Self)
            }
        }

        impl From<$ty> for $var_int {
            fn from(value: $ty) -> Self {
                Self(value)
            }
        }

        impl From<$var_int> for $ty {
            fn from(value: $var_int) -> Self {
                value.0
            }
        }

        impl std::ops::Add<$ty> for $var_int {
            type Output = $var_int;

            fn add(self, rhs: $ty) -> Self::Output {
                Self(self.0 + rhs)
            }
        }
    };
}

pub(super) use impl_var_int;
