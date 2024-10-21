pub mod boundingbox;
pub mod distance;
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
