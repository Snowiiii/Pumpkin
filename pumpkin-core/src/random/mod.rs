use std::{
    sync::atomic::{AtomicU64, Ordering},
    time,
};

use legacy_rand::{LegacyRand, LegacySplitter};
use xoroshiro128::{Xoroshiro, XoroshiroSplitter};

mod gaussian;
pub mod legacy_rand;
pub mod xoroshiro128;

static SEED_UNIQUIFIER: AtomicU64 = AtomicU64::new(8682522807148012u64);

pub fn get_seed() -> u64 {
    let seed = SEED_UNIQUIFIER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
            Some(val.wrapping_mul(1181783497276652981u64))
        })
        // We always return Some, so there will always be an Ok result
        .unwrap();

    let nanos = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let nano_upper = (nanos >> 8) as u64;
    let nano_lower = nanos as u64;
    seed ^ nano_upper ^ nano_lower
}

pub enum RandomGenerator {
    Xoroshiro(Xoroshiro),
    Legacy(LegacyRand),
}

impl RandomGenerator {
    #[inline]
    pub fn split(&mut self) -> Self {
        match self {
            Self::Xoroshiro(rand) => Self::Xoroshiro(rand.split()),
            Self::Legacy(rand) => Self::Legacy(rand.split()),
        }
    }

    #[inline]
    pub fn next_splitter(&mut self) -> RandomDeriver {
        match self {
            Self::Xoroshiro(rand) => RandomDeriver::Xoroshiro(rand.next_splitter()),
            Self::Legacy(rand) => RandomDeriver::Legacy(rand.next_splitter()),
        }
    }

    #[inline]
    pub fn next_i32(&mut self) -> i32 {
        match self {
            Self::Xoroshiro(rand) => rand.next_i32(),
            Self::Legacy(rand) => rand.next_i32(),
        }
    }

    #[inline]
    pub fn next_bounded_i32(&mut self, bound: i32) -> i32 {
        match self {
            Self::Xoroshiro(rand) => rand.next_bounded_i32(bound),
            Self::Legacy(rand) => rand.next_bounded_i32(bound),
        }
    }

    #[inline]
    pub fn next_inbetween_i32(&mut self, min: i32, max: i32) -> i32 {
        self.next_bounded_i32(max - min + 1) + min
    }

    #[inline]
    pub fn next_i64(&mut self) -> i64 {
        match self {
            Self::Xoroshiro(rand) => rand.next_i64(),
            Self::Legacy(rand) => rand.next_i64(),
        }
    }

    #[inline]
    pub fn next_bool(&mut self) -> bool {
        match self {
            Self::Xoroshiro(rand) => rand.next_bool(),
            Self::Legacy(rand) => rand.next_bool(),
        }
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        match self {
            Self::Xoroshiro(rand) => rand.next_f32(),
            Self::Legacy(rand) => rand.next_f32(),
        }
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        match self {
            Self::Xoroshiro(rand) => rand.next_f64(),
            Self::Legacy(rand) => rand.next_f64(),
        }
    }

    #[inline]
    pub fn next_gaussian(&mut self) -> f64 {
        match self {
            Self::Xoroshiro(rand) => rand.next_gaussian(),
            Self::Legacy(rand) => rand.next_gaussian(),
        }
    }

    #[inline]
    pub fn next_triangular(&mut self, mode: f64, deviation: f64) -> f64 {
        deviation.mul_add(self.next_f64() - self.next_f64(), mode)
    }

    #[inline]
    pub fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_i64();
        }
    }

    #[inline]
    pub fn next_inbetween_i32_exclusive(&mut self, min: i32, max: i32) -> i32 {
        min + self.next_bounded_i32(max - min)
    }
}

#[derive(Clone)]
pub enum RandomDeriver {
    Xoroshiro(XoroshiroSplitter),
    Legacy(LegacySplitter),
}

impl RandomDeriver {
    #[inline]
    pub fn split_string(&self, seed: &str) -> RandomGenerator {
        match self {
            Self::Xoroshiro(deriver) => RandomGenerator::Xoroshiro(deriver.split_string(seed)),
            Self::Legacy(deriver) => RandomGenerator::Legacy(deriver.split_string(seed)),
        }
    }

    #[inline]
    pub fn split_u64(&self, seed: u64) -> RandomGenerator {
        match self {
            Self::Xoroshiro(deriver) => RandomGenerator::Xoroshiro(deriver.split_u64(seed)),
            Self::Legacy(deriver) => RandomGenerator::Legacy(deriver.split_u64(seed)),
        }
    }

    #[inline]
    pub fn split_pos(&self, x: i32, y: i32, z: i32) -> RandomGenerator {
        match self {
            Self::Xoroshiro(deriver) => RandomGenerator::Xoroshiro(deriver.split_pos(x, y, z)),
            Self::Legacy(deriver) => RandomGenerator::Legacy(deriver.split_pos(x, y, z)),
        }
    }
}

pub trait RandomImpl {
    fn from_seed(seed: u64) -> Self;

    fn split(&mut self) -> Self;

    fn next_splitter(&mut self) -> impl RandomDeriverImpl;

    fn next_i32(&mut self) -> i32;

    fn next_bounded_i32(&mut self, bound: i32) -> i32;

    fn next_inbetween_i32(&mut self, min: i32, max: i32) -> i32 {
        self.next_bounded_i32(max - min + 1) + min
    }

    fn next_i64(&mut self) -> i64;

    fn next_bool(&mut self) -> bool;

    fn next_f32(&mut self) -> f32;

    fn next_f64(&mut self) -> f64;

    fn next_gaussian(&mut self) -> f64;

    fn next_triangular(&mut self, mode: f64, deviation: f64) -> f64 {
        mode + deviation * (self.next_f64() - self.next_f64())
    }

    fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_i64();
        }
    }

    fn next_inbetween_i32_exclusive(&mut self, min: i32, max: i32) -> i32 {
        min + self.next_bounded_i32(max - min)
    }
}

pub trait RandomDeriverImpl {
    fn split_string(&self, seed: &str) -> impl RandomImpl;

    fn split_u64(&self, seed: u64) -> impl RandomImpl;

    fn split_pos(&self, x: i32, y: i32, z: i32) -> impl RandomImpl;
}

pub fn hash_block_pos(x: i32, y: i32, z: i32) -> i64 {
    let l = (x.wrapping_mul(3129871) as i64) ^ ((z as i64).wrapping_mul(116129781i64)) ^ (y as i64);
    let l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861i64)
        .wrapping_add(l.wrapping_mul(11i64));
    l >> 16
}

pub fn java_string_hash(string: &str) -> i32 {
    // All byte values of latin1 align with
    // the values of U+0000 - U+00FF making this code
    // equivalent to both java hash implementations

    let mut result = 0i32;

    for char_encoding in string.encode_utf16() {
        result = 31i32
            .wrapping_mul(result)
            .wrapping_add(char_encoding as i32);
    }
    result
}

#[cfg(test)]
mod tests {

    use crate::random::java_string_hash;

    use super::hash_block_pos;

    #[test]
    fn block_position_hash() {
        let values: [((i32, i32, i32), i64); 8] = [
            ((0, 0, 0), 0),
            ((1, 1, 1), 60311958971344),
            ((4, 4, 4), 120566413180880),
            ((25, 25, 25), 111753446486209),
            ((676, 676, 676), 75210837988243),
            ((458329, 458329, 458329), -43764888250),
            ((-387008604, -387008604, -387008604), 8437923733503),
            ((176771161, 176771161, 176771161), 18421337580760),
        ];

        for ((x, y, z), value) in values {
            assert_eq!(hash_block_pos(x, y, z), value);
        }
    }

    #[test]
    fn test_java_string_hash() {
        let values = [
            ("", 0),
            ("1", 49),
            ("TEST", 2571410),
            ("TEST1", 79713759),
            ("TEST0123456789", 506557463),
            (
                " !\"#$%&'()*+,-./0123456789:\
                ;<=>?@ABCDEFGHIJKLMNOPQRST\
                UVWXYZ[\\]^_`abcdefghijklm\
                nopqrstuvwxyz{|}~¡¢£¤¥¦§¨©\
                ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄ\
                ÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞ\
                ßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþ",
                -1992287231i32,
            ),
            ("求同存异", 847053876),
            // This might look weird because hebrew is text is right to left
            ("אבְּרֵאשִׁ֖ית בָּרָ֣א אֱלֹהִ֑ים אֵ֥ת הַשָּׁמַ֖יִם וְאֵ֥ת הָאָֽרֶץ:", 1372570871),
            ("संस्कृत-", 1748614838),
            ("minecraft:offset", -920384768i32),
        ];

        for (string, value) in values {
            assert_eq!(java_string_hash(string), value);
        }
    }
}
