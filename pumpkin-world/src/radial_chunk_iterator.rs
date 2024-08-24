use crate::coordinates::ChunkCoordinates;

pub struct RadialIterator {
    radius: u32,
    direction: usize,
    current: ChunkCoordinates,
    step_size: i32,
    steps_taken: u32,
    steps_in_direction: i32,
}

impl RadialIterator {
    pub fn new(radius: u32) -> Self {
        RadialIterator {
            radius,
            direction: 0,
            current: ChunkCoordinates { x: 0, z: 0 },
            step_size: 1,
            steps_taken: 0,
            steps_in_direction: 0,
        }
    }
}

impl Iterator for RadialIterator {
    type Item = ChunkCoordinates;

    fn next(&mut self) -> Option<Self::Item> {
        if self.steps_taken >= self.radius * self.radius * 4 {
            return None;
        }

        let result = self.current;

        self.steps_in_direction += 1;

        // Move in the current direction
        match self.direction {
            0 => self.current.x += 1, // East
            1 => self.current.z += 1, // North
            2 => self.current.x -= 1, // West
            3 => self.current.z -= 1, // South
            _ => {}
        }

        if self.steps_in_direction >= self.step_size {
            self.direction = (self.direction + 1) % 4;
            self.steps_in_direction = 0;

            // Increase step size after completing two directions
            if self.direction == 0 || self.direction == 2 {
                self.step_size += 1;
            }
        }

        self.steps_taken += 1;
        Some(result)
    }
}
