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
