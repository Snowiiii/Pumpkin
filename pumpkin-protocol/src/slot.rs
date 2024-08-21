use crate::VarInt;
use pumpkin_world::item::Item;
use serde::{de::{self, SeqAccess, Visitor}, Deserialize, Serialize, Serializer};
use serde::ser::SerializeSeq;

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

impl Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.item_count == 0.into() {
            let mut s = serializer.serialize_seq(Some(1))?;
            s.serialize_element(&self.item_count)?;
            s.end()
        } else {
            match (&self.num_components_to_add,&self.num_components_to_remove) {
                (Some(to_add),Some(to_remove)) => {
                    let mut s = serializer.serialize_seq(Some(6))?;
                    s.serialize_element(&self.item_count)?;
                    s.serialize_element(self.item_id.as_ref().unwrap())?;
                    s.serialize_element(to_add)?;
                    s.serialize_element(to_remove)?;
                    s.serialize_element(self.components_to_add.as_ref().unwrap())?;
                    s.serialize_element(self.components_to_remove.as_ref().unwrap())?;
                    s.end()
                }
                (None, Some(to_remove)) => {
                    let mut s = serializer.serialize_seq(Some(5))?;
                    s.serialize_element(&self.item_count)?;
                    s.serialize_element(self.item_id.as_ref().unwrap())?;
                    s.serialize_element(&VarInt(0))?;
                    s.serialize_element(to_remove)?;
                    s.serialize_element(self.components_to_remove.as_ref().unwrap())?;
                    s.end()
                }
                (Some(to_add),None) => {
                    let mut s = serializer.serialize_seq(Some(5))?;
                    s.serialize_element(&self.item_count)?;
                    s.serialize_element(self.item_id.as_ref().unwrap())?;
                    s.serialize_element(to_add)?;
                    s.serialize_element(&VarInt(0))?;
                    s.serialize_element(self.components_to_add.as_ref().unwrap())?;
                    s.end()
                }
                (None,None) => {
                    let mut s = serializer.serialize_seq(Some(4))?;
                    s.serialize_element(&self.item_count)?;
                    s.serialize_element(&self.item_id.as_ref().unwrap())?;
                    s.serialize_element(&VarInt(0))?;
                    s.serialize_element(&VarInt(0))?;
                    s.end()
                }
            }
            
        }
    }
}

impl Slot {
    pub fn to_item(self) -> Option<Item> {
        let item_id = self.item_id?.0.try_into().unwrap();
        Some(Item {
            item_id,
            item_count: self.item_count.0.try_into().unwrap(),
        })
    }

    pub const fn empty() -> Self {
        Slot {
            item_count: VarInt(0),
            item_id: None,
            num_components_to_add: None,
            num_components_to_remove: None,
            components_to_add: None,
            components_to_remove: None,
        }
    }
}

impl From<&Item> for Slot {
    fn from(item: &Item) -> Self {
        Slot {
            item_count: item.item_count.into(),
            item_id: Some(item.item_id.into()),
            // TODO: add these
            num_components_to_add:None, 
            num_components_to_remove: None,
            components_to_add: None,
            components_to_remove: None,
        }
    }
}