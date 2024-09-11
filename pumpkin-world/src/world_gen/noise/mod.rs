mod perlin;
mod simplex;

pub fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

pub fn lerp2(delta_x: f64, delta_y: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

pub fn lerp3(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    x0y0z0: f64,
    x1y0z0: f64,
    x0y1z0: f64,
    x1y1z0: f64,
    x0y0z1: f64,
    x1y0z1: f64,
    x0y1z1: f64,
    x1y1z1: f64,
) -> f64 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(delta_x, delta_y, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

struct Gradient {
    x: i32,
    y: i32,
    z: i32,
}

const GRADIENTS: [Gradient; 16] = [
    Gradient { x: 1, y: 1, z: 0 },
    Gradient { x: -1, y: 1, z: 0 },
    Gradient { x: 1, y: -1, z: 0 },
    Gradient { x: -1, y: -1, z: 0 },
    Gradient { x: 1, y: 0, z: 1 },
    Gradient { x: -1, y: 0, z: 1 },
    Gradient { x: 1, y: 0, z: -1 },
    Gradient { x: -1, y: 0, z: -1 },
    Gradient { x: 0, y: 1, z: 1 },
    Gradient { x: 0, y: -1, z: 1 },
    Gradient { x: 0, y: 1, z: -1 },
    Gradient { x: 0, y: -1, z: -1 },
    Gradient { x: 1, y: 1, z: 0 },
    Gradient { x: 0, y: -1, z: 1 },
    Gradient { x: -1, y: 1, z: 0 },
    Gradient { x: 0, y: -1, z: -1 },
];

fn dot(gradient: &Gradient, x: f64, y: f64, z: f64) -> f64 {
    gradient.x as f64 * x + gradient.y as f64 * y + gradient.z as f64 * z
}
