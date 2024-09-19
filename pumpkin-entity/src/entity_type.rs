// TODO
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum EntityType {
    Item = 58,
    Zombie = 124,
    Player = 128,
}

impl EntityType {
    pub const fn gravity(&self) -> f64 {
        use EntityType::*;
        match self {
            Item => 0.04,
            _ => todo!(),
        }
    }
}
