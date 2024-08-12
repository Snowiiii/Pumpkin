use serde::Serialize;

#[derive(Clone)]
pub struct UUID(pub uuid::Uuid);

impl Serialize for UUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes())
    }
}
