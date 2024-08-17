use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize,
};

use crate::VarInt;

#[derive(Debug, Clone)]
pub struct Slot {
    item_count: VarInt,
    item_id: Option<VarInt>,
    num_components_to_add: Option<VarInt>,
    num_components_to_remove: Option<VarInt>,
    components_to_add: Option<Vec<(VarInt, ())>>, // The second type depends on the varint
    components_to_remove: Option<Vec<VarInt>>,
}

impl<'de> Deserialize<'de> for Slot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VarIntVisitor;
        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = Slot;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let item_count = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                if item_count.0 == 0 {
                    return Ok(Slot {
                        item_count: 0.into(),
                        item_id: None,
                        num_components_to_add: None,
                        num_components_to_remove: None,
                        components_to_add: None,
                        components_to_remove: None,
                    });
                }
                let item_id = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                let num_components_to_add = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                let num_components_to_remove = seq
                    .next_element::<VarInt>()?
                    .ok_or(de::Error::custom("Failed to decode VarInt"))?;
                if num_components_to_add.0 != 0 || num_components_to_remove.0 != 0 {
                    return Err(de::Error::custom(
                        "Slot components are currently unsupported",
                    ));
                }

                Ok(Slot {
                    item_count,
                    item_id: Some(item_id),
                    num_components_to_add: Some(num_components_to_add),
                    num_components_to_remove: Some(num_components_to_remove),
                    components_to_add: None,
                    components_to_remove: None,
                })
            }
        }

        deserializer.deserialize_seq(VarIntVisitor)
    }
}
