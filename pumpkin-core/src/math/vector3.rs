use std::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::Float;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Math + Copy + Default> Default for Vector3<T> {
    fn default() -> Self {
        Vector3 {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T: Math + Copy> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Vector3 { x, y, z }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn horizontal_length_squared(&self) -> T {
        (self.x * self.x) + (self.z * self.z)
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

impl<T: Math + Copy + std::iter::Sum + Default> std::iter::Sum for Vector3<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut x = T::default();
        let mut y = T::default();
        let mut z = T::default();
        for vec in iter {
            x = x + vec.x;
            y = y + vec.y;
            z = z + vec.z;
        }
        Vector3 { x, y, z }
    }
}

impl<T: Math + Copy + Float> Vector3<T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn horizontal_length(&self) -> T {
        self.horizontal_length_squared().sqrt()
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
    + Neg<Output = Self>
    + Add<Output = Self>
    + Div<Output = Self>
    + Sub<Output = Self>
    + Sized
{
}
impl Math for f64 {}
impl Math for f32 {}
impl Math for i32 {}
impl Math for i64 {}
