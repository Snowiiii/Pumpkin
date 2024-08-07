use num_derive::ToPrimitive;

// TODO
#[derive(ToPrimitive, Clone)]
pub enum EntityType {
    Zombie = 124,
    Player = 128,
}
