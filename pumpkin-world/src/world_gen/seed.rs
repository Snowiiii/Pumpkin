use std::hash::{DefaultHasher, Hash, Hasher};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Seed(pub i64);

impl From<&str> for Seed {
    fn from(value: &str) -> Self {
        // TODO replace with a deterministic hasher (the same as vanilla?)
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);

        // TODO use cast_signed once the feature is stabilized.
        Self(hasher.finish() as i64)
    }
}
