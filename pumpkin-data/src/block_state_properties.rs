/// a lookup-array for the block property values of each block state
#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_state_properties.rs"));

#[cfg(clippy)]
pub static BLOCK_STATE_PROPERTY_VALUES: [&[&str]; 0] = [];
