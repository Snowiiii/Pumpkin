use crate::slot::Slot;
use crate::VarInt;
use serde::de::SeqAccess;
use serde::{de, Deserialize};

#[derive(Debug)]
pub struct SClickContainer {
    pub window_id: u8,
    pub state_id: VarInt,
    pub slot: i16,
    pub button: i8,
    pub mode: VarInt,
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
                    window_id,
                    state_id,
                    slot,
                    button,
                    mode,
                    length_of_array,
                    array_of_changed_slots,
                    carried_item,
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}
