use bytes::BufMut;
use std::ops::{Add, Div, Mul, Sub};

use num_traits::Float;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, Default)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Math + Copy> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Vector3 { x, y, z }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn add(&self, other: &Vector3<T>) -> Self {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(&self, other: &Vector3<T>) -> Self {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn multiply(self, x: T, y: T, z: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y * y,
            z: self.z * z,
        }
    }
}

impl<T: Math + Copy + Float> Vector3<T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Vector3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

impl<T: Math + Copy> Mul<T> for Vector3<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<T: Math + Copy> Add for Vector3<T> {
    type Output = Vector3<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/*
impl<T: Math + Copy> Neg for Vector3<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
*/

impl<T> From<(T, T, T)> for Vector3<T> {
    #[inline(always)]
    fn from((x, y, z): (T, T, T)) -> Self {
        Vector3 { x, y, z }
    }
}

impl<T> From<Vector3<T>> for (T, T, T) {
    #[inline(always)]
    fn from(vector: Vector3<T>) -> Self {
        (vector.x, vector.y, vector.z)
    }
}

pub trait Math:
    Mul<Output = Self>
    //+ Neg<Output = Self>
    + Add<Output = Self>
    + Div<Output = Self>
    + Sub<Output = Self>
    + Sized
{
}
impl Math for i16 {}
impl Math for f64 {}
impl Math for f32 {}
impl Math for i32 {}
impl Math for i64 {}
impl Math for u8 {}

impl<'de> serde::Deserialize<'de> for Vector3<f32> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vector3Visitor;

        impl<'de> serde::de::Visitor<'de> for Vector3Visitor {
            type Value = Vector3<f32>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Vector<32>")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                if let Some(x) = seq.next_element::<f32>()? {
                    if let Some(y) = seq.next_element::<f32>()? {
                        if let Some(z) = seq.next_element::<f32>()? {
                            return Ok(Vector3::new(x, y, z));
                        }
                    }
                }
                Err(serde::de::Error::custom("Failed to read Vector<f32>"))
            }
        }

        deserializer.deserialize_seq(Vector3Visitor)
    }
}

impl<'de> serde::Deserialize<'de> for Vector3<f64> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vector3Visitor;

        impl<'de> serde::de::Visitor<'de> for Vector3Visitor {
            type Value = Vector3<f64>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Vector<f64>")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                if let Some(x) = seq.next_element::<f64>()? {
                    if let Some(y) = seq.next_element::<f64>()? {
                        if let Some(z) = seq.next_element::<f64>()? {
                            return Ok(Vector3::new(x, y, z));
                        }
                    }
                }
                Err(serde::de::Error::custom("Failed to read Vector<f64>"))
            }
        }

        deserializer.deserialize_seq(Vector3Visitor)
    }
}

impl serde::Serialize for Vector3<f32> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::new();
        buf.put_f32(self.x);
        buf.put_f32(self.y);
        buf.put_f32(self.z);
        serializer.serialize_bytes(&buf)
    }
}

impl serde::Serialize for Vector3<f64> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::new();
        buf.put_f64(self.x);
        buf.put_f64(self.y);
        buf.put_f64(self.z);
        serializer.serialize_bytes(&buf)
    }
}

impl serde::Serialize for Vector3<i16> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::new();
        buf.put_i16(self.x);
        buf.put_i16(self.y);
        buf.put_i16(self.z);
        serializer.serialize_bytes(&buf)
    }
}
