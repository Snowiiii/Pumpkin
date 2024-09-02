pub mod boundingbox;
pub mod position;
pub mod vector2;
pub mod vector3;

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
    a * a + b * b + c * c
}

/// Converts a world coordinate to the corresponding chunk-section coordinate.
// TODO: This proberbly should place not here
pub fn get_section_cord(coord: i32) -> i32 {
    coord >> 4
}
