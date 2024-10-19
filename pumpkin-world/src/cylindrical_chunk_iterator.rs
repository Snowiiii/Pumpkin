use pumpkin_core::math::vector2::Vector2;

#[derive(Debug, PartialEq)]
pub struct Cylindrical {
    pub center: Vector2<i32>,
    pub view_distance: i32,
}

impl Cylindrical {
    pub fn new(center: Vector2<i32>, view_distance: i32) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    #[allow(unused_variables)]
    pub fn for_each_changed_chunk(
        old_cylindrical: Cylindrical,
        new_cylindrical: Cylindrical,
        mut newly_included: impl FnMut(Vector2<i32>),
        mut just_removed: impl FnMut(Vector2<i32>),
        ignore: bool,
    ) {
        let min_x = old_cylindrical.left().min(new_cylindrical.left());
        let max_x = old_cylindrical.right().max(new_cylindrical.right());
        let min_z = old_cylindrical.bottom().min(new_cylindrical.bottom());
        let max_z = old_cylindrical.top().max(new_cylindrical.top());

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let old_is_within = if ignore {
                    false
                } else {
                    old_cylindrical.is_within_distance(x, z)
                };
                let new_is_within = if ignore {
                    true
                } else {
                    new_cylindrical.is_within_distance(x, z)
                };

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
        self.center.x - self.view_distance - 1
    }

    fn bottom(&self) -> i32 {
        self.center.z - self.view_distance - 1
    }

    fn right(&self) -> i32 {
        self.center.x + self.view_distance + 1
    }

    fn top(&self) -> i32 {
        self.center.z + self.view_distance + 1
    }

    #[allow(dead_code)]
    fn is_within_distance(&self, x: i32, z: i32) -> bool {
        let max_dist_squared = self.view_distance * self.view_distance;
        let max_dist = self.view_distance as i64;
        let dist_x = (x - self.center.x).abs().max(0) - (1);
        let dist_z = (z - self.center.z).abs().max(0) - (1);
        let dist_squared = dist_x.pow(2) + (max_dist.min(dist_z as i64) as i32).pow(2);
        dist_squared < max_dist_squared
    }
}
