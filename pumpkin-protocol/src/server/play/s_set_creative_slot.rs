use crate::slot::Slot;

#[derive(serde::Deserialize, Debug)]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: Slot,
}
