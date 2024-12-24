use pumpkin_core::random::{get_seed, java_string_hash, legacy_rand::LegacyRand, RandomImpl};

#[derive(Clone, Copy)]
pub struct Seed(pub u64);

impl From<&str> for Seed {
    fn from(value: &str) -> Self {
        let trimmed = value.trim();
        let value = if !trimmed.is_empty() {
            let i64_value = trimmed
                .parse::<i64>()
                .unwrap_or_else(|_| java_string_hash(trimmed) as i64);
            Some(i64_value as u64)
        } else {
            None
        };

        Seed(value.unwrap_or_else(|| LegacyRand::from_seed(get_seed()).next_i64() as u64))
    }
}
