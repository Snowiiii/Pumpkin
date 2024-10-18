use super::vector3::Vector3;

pub fn distance(p1: &Vector3<f64>, p2: &Vector3<f64>) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    let dz = p1.z - p2.z;

    dz.mul_add(dz, dx.mul_add(dx, dy * dy)).sqrt()
}
