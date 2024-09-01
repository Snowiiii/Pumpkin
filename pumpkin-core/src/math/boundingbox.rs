use super::{position::WorldPosition, vector3::Vector3};

pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn from_block(position: &WorldPosition) -> Self {
        let position = position.0;
        Self {
            min_x: position.x as f64,
            min_y: position.y as f64,
            min_z: position.z as f64,
            max_x: (position.x as f64) + 1.0,
            max_y: (position.y as f64) + 1.0,
            max_z: (position.z as f64) + 1.0,
        }
    }

    pub fn squared_magnitude(&self, pos: Vector3<f64>) -> f64 {
        let d = f64::max(f64::max(self.min_x - pos.x, pos.x - self.max_x), 0.0);
        let e = f64::max(f64::max(self.min_y - pos.y, pos.y - self.max_y), 0.0);
        let f = f64::max(f64::max(self.min_z - pos.z, pos.z - self.max_z), 0.0);
        super::squared_magnitude(d, e, f)
    }
}
