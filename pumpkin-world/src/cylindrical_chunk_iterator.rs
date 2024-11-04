use pumpkin_core::math::vector2::Vector2;

#[derive(Debug, PartialEq)]
pub struct Cylindrical {
    pub center: Vector2<i32>,
    pub view_distance: u8,
}

impl Cylindrical {
    pub fn new(center: Vector2<i32>, view_distance: u8) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    pub fn for_each_changed_chunk(
        old_cylindrical: Cylindrical,
        new_cylindrical: Cylindrical,
        mut newly_included: impl FnMut(Vector2<i32>),
        mut just_removed: impl FnMut(Vector2<i32>),
    ) {
        let min_x = old_cylindrical.left().min(new_cylindrical.left());
        let max_x = old_cylindrical.right().max(new_cylindrical.right());
        let min_z = old_cylindrical.bottom().min(new_cylindrical.bottom());
        let max_z = old_cylindrical.top().max(new_cylindrical.top());

        //log::debug!("{:?} {:?}", old_cylindrical, new_cylindrical);
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let old_is_within = old_cylindrical.is_within_distance(x, z);
                let new_is_within = new_cylindrical.is_within_distance(x, z);

                //log::debug!("{}, {}: {} {}", x, z, old_is_within, new_is_within);

                if old_is_within != new_is_within {
                    if new_is_within {
                        newly_included(Vector2::new(x, z));
                    } else {
                        just_removed(Vector2::new(x, z));
                    }
                }
            }
        }
    }

    fn left(&self) -> i32 {
        self.center.x - self.view_distance as i32 - 1
    }

    fn bottom(&self) -> i32 {
        self.center.z - self.view_distance as i32 - 1
    }

    fn right(&self) -> i32 {
        self.center.x + self.view_distance as i32 + 1
    }

    fn top(&self) -> i32 {
        self.center.z + self.view_distance as i32 + 1
    }

    fn is_within_distance(&self, x: i32, z: i32) -> bool {
        let rel_x = ((x - self.center.x).abs() - 1).max(0);
        let rel_z = ((z - self.center.z).abs() - 1).max(0);

        let max_leg = (rel_x.max(rel_z) - 1).max(0) as i64;
        let min_leg = rel_x.min(rel_z) as i64;

        let hyp_sqr = max_leg * max_leg + min_leg * min_leg;
        hyp_sqr < (self.view_distance * self.view_distance) as i64
    }

    /// Returns an iterator of all chunks within this cylinder
    pub fn all_chunks_within(&self) -> Vec<Vector2<i32>> {
        // This is a naive implementation: start with square and cut out ones that dont fit
        let mut all_chunks = Vec::new();
        for x in self.left()..=self.right() {
            for z in self.bottom()..=self.top() {
                all_chunks.push(Vector2::new(x, z));
            }
        }
        all_chunks
            .into_iter()
            .filter(|chunk| self.is_within_distance(chunk.x, chunk.z))
            .collect()
    }
}

#[cfg(test)]
mod test {

    use super::Cylindrical;
    use pumpkin_core::math::vector2::Vector2;

    #[test]
    fn test_bounds() {
        let cylinder = Cylindrical::new(Vector2::new(0, 0), 10);
        for chunk in cylinder.all_chunks_within() {
            assert!(chunk.x >= cylinder.left() && chunk.x <= cylinder.right());
            assert!(chunk.z >= cylinder.bottom() && chunk.z <= cylinder.top());
        }

        for x in (cylinder.left() - 2)..=(cylinder.right() + 2) {
            for z in (cylinder.bottom() - 2)..=(cylinder.top() + 2) {
                if cylinder.is_within_distance(x, z) {
                    assert!(x >= cylinder.left() && x <= cylinder.right());
                    assert!(z >= cylinder.bottom() && z <= cylinder.top());
                }
            }
        }
    }
}
