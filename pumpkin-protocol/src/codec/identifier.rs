use std::num::NonZeroUsize;

use bytes::{Buf, BufMut};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::bytebuf::{ByteBuf, ByteBufMut};

use super::{Codec, DecodeError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn vanilla(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}
impl Codec<Self> for Identifier {
    /// The maximum number of bytes a `Identifer` is the same as for a normal String.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(i16::MAX as usize) };

    fn written_size(&self) -> usize {
        todo!()
    }

    fn encode(&self, write: &mut impl BufMut) {
        write.put_string_len(&self.to_string(), Self::MAX_SIZE.get());
    }

    fn decode(read: &mut impl Buf) -> Result<Self, DecodeError> {
        let identifer = read
            .try_get_string_len(Self::MAX_SIZE.get())
            .map_err(|_| DecodeError::Incomplete)?;
        match identifer.split_once(":") {
            Some((namespace, path)) => Ok(Identifier {
                namespace: namespace.to_string(),
                path: path.to_string(),
            }),
            None => Err(DecodeError::Incomplete),
        }
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierVisitor;

        impl Visitor<'_> for IdentifierVisitor {
            type Value = Identifier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Identifier (namespace:path)")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_str<E>(self, identifer: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match identifer.split_once(":") {
                    Some((namespace, path)) => Ok(Identifier {
                        namespace: namespace.to_string(),
                        path: path.to_string(),
                    }),
                    None => Err(serde::de::Error::custom("Identifier can't be split")),
                }
            }
        }
        deserializer.deserialize_str(IdentifierVisitor)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}
