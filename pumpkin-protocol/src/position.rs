use serde::Deserialize;

pub struct WorldPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl<'de> Deserialize<'de> for WorldPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'a> serde::de::Visitor<'a> for Visitor {
            type Value = WorldPosition;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("An i64 int")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(WorldPosition {
                    x: (v >> 38) as i32,
                    y: (v << 52 >> 52) as i32,
                    z: (v << 26 >> 38) as i32,
                })
            }
        }
        deserializer.deserialize_i64(Visitor)
    }
}
