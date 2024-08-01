use crate::entity::player::Player;

pub struct World {
    pub players: Vec<Player>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }

    pub fn get_region_file(x: f32, z: f32) -> String {
        let result: f32 = 7.0 / 32.0;
        dbg!("{}", result.floor());

        let region_file_name = format!("r.{}.{}.mca", (x / 32.0).floor(), (z / 32.0).floor());
        region_file_name
    }

    pub fn get_intern_coords(minecraft_x: i32, minecraft_z: i32) -> (i32, i32) {
        let result_x = minecraft_x.rem_euclid(32);
        let result_z = minecraft_z.rem_euclid(32);

        (result_x, result_z)
    }
}
