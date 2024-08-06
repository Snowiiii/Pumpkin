use num_derive::ToPrimitive;

// todo
#[derive(ToPrimitive, Clone)]
pub enum EntityType {
    Zombie = 124,
    Player = 128,
}
