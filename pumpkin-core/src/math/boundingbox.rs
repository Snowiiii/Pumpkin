use super::{position::WorldPosition, vector3::Vector3};

#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub minimum: Vector3<f64>,
    pub maximum: Vector3<f64>,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            minimum: Vector3 {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            maximum: Vector3 {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }

    pub fn from_block(position: &WorldPosition) -> Self {
        let position = position.0;
        let minimum = Vector3 {
            x: position.x as f64,
            y: position.y as f64,
            z: position.z as f64,
        };
        Self {
            minimum,
            maximum: minimum.add(&Vector3 {
                x: 1.,
                y: 1.,
                z: 1.,
            }),
        }
    }

    pub fn squared_magnitude(&self, pos: Vector3<f64>) -> f64 {
        let d = f64::max(
            f64::max(self.minimum.x - pos.x, pos.x - self.maximum.x),
            0.0,
        );
        let e = f64::max(
            f64::max(self.minimum.y - pos.y, pos.y - self.maximum.y),
            0.0,
        );
        let f = f64::max(
            f64::max(self.minimum.z - pos.z, pos.z - self.maximum.z),
            0.0,
        );
        super::squared_magnitude(d, e, f)
    }

    pub fn offset(&mut self, vector3: Vector3<f64>) {
        self.minimum = self.minimum.add(&vector3);
        self.maximum = self.maximum.add(&vector3);
    }
}
