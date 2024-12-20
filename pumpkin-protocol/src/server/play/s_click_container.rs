use crate::slot::Slot;
use crate::VarInt;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::server_packet;
use serde::de::SeqAccess;
use serde::{de, Deserialize};

#[server_packet("play:container_click")]
pub struct SClickContainer {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slot: i16,
    pub button: i8,
    pub mode: SlotActionType,
    pub length_of_array: VarInt,
    pub array_of_changed_slots: Vec<(i16, Slot)>,
    pub carried_item: Slot,
}

impl<'de> Deserialize<'de> for SClickContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SClickContainer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let window_id = seq
                    .next_element::<u8>()?
                    .ok_or(de::Error::custom("Failed to decode u8"))?;
                let state_id = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;

                let slot = seq
                    .next_element::<i16>()?
                    .ok_or(de::Error::custom("Failed to decode i16"))?;
                let button = seq
                    .next_element::<i8>()?
                    .ok_or(de::Error::custom("Failed to decode i8"))?;
                let mode = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                let length_of_array = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                let mut array_of_changed_slots = vec![];
                for _ in 0..length_of_array.0 {
                    let slot_number = seq
                        .next_element::<i16>()?
                        .ok_or(de::Error::custom("Unable to parse slot"))?;
                    let slot = seq
                        .next_element::<Slot>()?
                        .ok_or(de::Error::custom("Unable to parse item"))?;
                    array_of_changed_slots.push((slot_number, slot));
                }

                let carried_item = seq
                    .next_element::<Slot>()?
                    .ok_or(de::Error::custom("Failed to decode carried item"))?;

                Ok(SClickContainer {
                    window_id: window_id.into(),
                    state_id,
                    slot,
                    button,
                    mode: SlotActionType::from_i32(mode.0)
                        .expect("Invalid Slot action, TODO better error handling ;D"),
                    length_of_array,
                    array_of_changed_slots,
                    carried_item,
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

#[derive(Deserialize, FromPrimitive)]
pub enum SlotActionType {
    /// Performs a normal slot click. This can pickup or place items in the slot, possibly merging the cursor stack into the slot, or swapping the slot stack with the cursor stack if they can't be merged.
    Pickup,
    /// Performs a shift-click. This usually quickly moves items between the player's inventory and the open screen handler.
    QuickMove,
    /// Exchanges items between a slot and a hotbar slot. This is usually triggered by the player pressing a 1-9 number key while hovering over a slot.
    /// When the action type is swap, the click data is the hotbar slot to swap with (0-8).
    Swap,
    /// Clones the item in the slot. Usually triggered by middle clicking an item in creative mode.
    Clone,
    /// Throws the item out of the inventory. This is usually triggered by the player pressing Q while hovering over a slot, or clicking outside the window.
    /// When the action type is throw, the click data determines whether to throw a whole stack (1) or a single item from that stack (0).
    Throw,
    /// Drags items between multiple slots. This is usually triggered by the player clicking and dragging between slots.
    /// This action happens in 3 stages. Stage 0 signals that the drag has begun, and stage 2 signals that the drag has ended. In between multiple stage 1s signal which slots were dragged on.
    QuickCraft,
    /// Replenishes the cursor stack with items from the screen handler. This is usually triggered by the player double clicking
    PickupAll,
}
