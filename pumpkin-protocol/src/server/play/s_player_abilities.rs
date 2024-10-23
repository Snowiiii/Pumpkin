use serde::Deserialize;

//The vanilla client sends this packet when the player starts/stops flying. Bitmask 0x02 is set when the player is flying.

#[derive(Deserialize)]
pub struct SPlayerAbilities {
    pub flags: i8,
}
