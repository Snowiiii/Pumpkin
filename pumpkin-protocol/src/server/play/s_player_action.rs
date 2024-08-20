use num_derive::FromPrimitive;
use pumpkin_macros::packet;

use crate::{position::WorldPosition, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x24)]
pub struct SPlayerAction {
    pub status: VarInt,
    pub location: WorldPosition,
    pub face: u8,
    pub sequence: VarInt,
}

#[derive(FromPrimitive)]
pub enum Status {
    /// Sent when the player starts digging a block. If the block was instamined or the player is in creative mode, the client will not send Status = Finished digging, and will assume the server completed the destruction. To detect this, it is necessary to calculate the block destruction speed server-side.
    StartedDigging = 0,
    /// Sent when the player lets go of the Mine Block key (default: left click). Face is always set to -Y.
    CancelledDigging,
    /// Sent when the client thinks it is finished.
    FinishedDigging,
    /// Triggered by using the Drop Item key (default: Q) with the modifier to drop the entire selected stack (default: Control or Command, depending on OS). Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    DropItemStack,
    /// Triggered by using the Drop Item key (default: Q). Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    DropItem,
    /// I didn't make that up
    /// Indicates that the currently held item should have its state updated such as eating food, pulling back bows, using buckets, etc. Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    ShootArrowOrFinishEating,
    /// Used to swap or assign an item to the second hand. Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.  
    SwapItem,
}
