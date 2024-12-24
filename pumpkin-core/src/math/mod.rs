use num_traits::PrimInt;

pub mod boundingbox;
pub mod position;
pub mod vector2;
pub mod vector3;
pub mod voxel_shape;

pub fn wrap_degrees(var: f32) -> f32 {
    let mut var1 = var % 360.0;
    if var1 >= 180.0 {
        var1 -= 360.0;
    }

    if var1 < -180.0 {
        var1 += 360.0;
    }

    var1
}

pub fn squared_magnitude(a: f64, b: f64, c: f64) -> f64 {
    c.mul_add(c, a.mul_add(a, b * b))
}

pub fn magnitude(a: f64, b: f64, c: f64) -> f64 {
    squared_magnitude(a, b, c).sqrt()
}

/// Converts a world coordinate to the corresponding chunk-section coordinate.
// TODO: This proberbly should place not here
pub const fn get_section_cord(coord: i32) -> i32 {
    coord >> 4
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

/// Maximum return value: 31
pub const fn ceil_log2(value: u32) -> u8 {
    let value = if value.is_power_of_two() {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };

    MULTIPLY_DE_BRUIJN_BIT_POSITION[(((value as usize) * 125613361) >> 27) & 31]
}

/// Maximum return value: 30
pub const fn floor_log2(value: u32) -> u8 {
    ceil_log2(value) - if value.is_power_of_two() { 0 } else { 1 }
}

pub const fn smallest_encompassing_power_of_two(value: u32) -> u32 {
    let mut i = value - 1;
    i |= i >> 1;
    i |= i >> 2;
    i |= i >> 4;
    i |= i >> 8;
    i |= i >> 16;
    i + 1
}

#[inline]
pub fn floor_div<T>(x: T, y: T) -> T
where
    T: PrimInt + From<i8>,
{
    let div = x / y;
    if (x ^ y) < 0.into() && div * y != x {
        div - 1.into()
    } else {
        div
    }
}

#[inline]
pub fn floor_mod<T>(x: T, y: T) -> T
where
    T: PrimInt + From<i8>,
{
    let rem = x % y;
    if (x ^ y) < 0.into() && rem != 0.into() {
        rem + y
    } else {
        rem
    }
}
