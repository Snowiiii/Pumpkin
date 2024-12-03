use super::{
    gaussian::GaussianGenerator, hash_block_pos, java_string_hash, RandomDeriverImpl, RandomImpl,
};

pub struct LegacyRand {
    seed: u64,
    internal_next_gaussian: Option<f64>,
}

impl LegacyRand {
    fn next_random(&mut self) -> i64 {
        let l = self.seed as i64;
        let m = l.wrapping_mul(0x5DEECE66D).wrapping_add(11) & 0xFFFFFFFFFFFF;
        self.seed = m as u64;
        m
    }

    fn next(&mut self, bits: u64) -> i32 {
        (self.next_random() >> (48 - bits)) as i32
    }
}

impl GaussianGenerator for LegacyRand {
    fn stored_next_gaussian(&self) -> Option<f64> {
        self.internal_next_gaussian
    }

    fn set_stored_next_gaussian(&mut self, value: Option<f64>) {
        self.internal_next_gaussian = value;
    }
}

impl RandomImpl for LegacyRand {
    fn from_seed(seed: u64) -> Self {
        LegacyRand {
            seed: (seed ^ 0x5DEECE66D) & 0xFFFFFFFFFFFF,
            internal_next_gaussian: None,
        }
    }

    fn split(&mut self) -> Self {
        LegacyRand::from_seed(self.next_i64() as u64)
    }

    fn next_i32(&mut self) -> i32 {
        self.next(32)
    }

    fn next_i64(&mut self) -> i64 {
        let i = self.next_i32();
        let j = self.next_i32();
        ((i as i64) << 32).wrapping_add(j as i64)
    }

    fn next_f32(&mut self) -> f32 {
        self.next(24) as f32 * 5.9604645E-8f32
    }

    fn next_f64(&mut self) -> f64 {
        let i = self.next(26);
        let j = self.next(27);
        let l = ((i as i64) << 27).wrapping_add(j as i64);
        l as f64 * 1.110223E-16f32 as f64
    }

    fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }

    #[allow(refining_impl_trait)]
    fn next_splitter(&mut self) -> LegacySplitter {
        LegacySplitter::new(self.next_i64() as u64)
    }

    fn next_gaussian(&mut self) -> f64 {
        self.calculate_gaussian()
    }

    fn next_bounded_i32(&mut self, bound: i32) -> i32 {
        if (bound & bound.wrapping_sub(1)) == 0 {
            ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32
        } else {
            loop {
                let i = self.next(31);
                let j = i % bound;
                if (i.wrapping_sub(j).wrapping_add(bound.wrapping_sub(1))) >= 0 {
                    return j;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct LegacySplitter {
    seed: u64,
}

impl LegacySplitter {
    fn new(seed: u64) -> Self {
        LegacySplitter { seed }
    }
}

#[allow(refining_impl_trait)]
impl RandomDeriverImpl for LegacySplitter {
    fn split_u64(&self, seed: u64) -> LegacyRand {
        LegacyRand::from_seed(seed)
    }

    fn split_string(&self, seed: &str) -> LegacyRand {
        let string_hash = java_string_hash(seed);
        LegacyRand::from_seed((string_hash as u64) ^ self.seed)
    }

    fn split_pos(&self, x: i32, y: i32, z: i32) -> LegacyRand {
        let pos_hash = hash_block_pos(x, y, z);
        LegacyRand::from_seed((pos_hash as u64) ^ self.seed)
    }
}

#[cfg(test)]
mod test {
    use crate::random::{legacy_rand::LegacySplitter, RandomDeriverImpl, RandomImpl};

    use super::LegacyRand;

    #[test]
    fn test_next_i32() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [
            -1155484576,
            -723955400,
            1033096058,
            -1690734402,
            -1557280266,
            1327362106,
            -1930858313,
            502539523,
            -1728529858,
            -938301587,
        ];

        for value in values {
            assert_eq!(rand.next_i32(), value);
        }
    }

    #[test]
    fn test_next_bounded_i32() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [0, 13, 4, 2, 5, 8, 11, 6, 9, 14];

        for value in values {
            assert_eq!(rand.next_bounded_i32(0xf), value);
        }

        let mut rand = LegacyRand::from_seed(0);
        for _ in 0..10 {
            assert_eq!(rand.next_bounded_i32(1), 0);
        }

        let mut rand = LegacyRand::from_seed(0);
        let values = [1, 1, 0, 1, 1, 0, 1, 0, 1, 1];
        for value in values {
            assert_eq!(rand.next_bounded_i32(2), value);
        }
    }

    #[test]
    fn test_next_inbetween_i32() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [1, 5, 2, 12, 12, 6, 12, 10, 4, 3];

        for value in values {
            assert_eq!(rand.next_inbetween_i32(1, 12), value);
        }
    }

    #[test]
    fn test_next_inbetween_exclusive_i32() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [1, 7, 9, 6, 7, 3, 3, 7, 3, 1];

        for value in values {
            assert_eq!(rand.next_inbetween_i32_exclusive(1, 12), value);
        }
    }

    #[test]
    fn test_next_f64() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [
            0.730967787376657,
            0.24053641567148587,
            0.6374174253501083,
            0.5504370051176339,
            0.5975452777972018,
            0.3332183994766498,
            0.3851891847407185,
            0.984841540199809,
            0.8791825178724801,
            0.9412491794821144,
        ];

        for value in values {
            assert_eq!(rand.next_f64(), value);
        }
    }

    #[test]
    fn test_next_f32() {
        let mut rand = LegacyRand::from_seed(0);

        let values: [f32; 10] = [
            0.73096776, 0.831441, 0.24053639, 0.6063452, 0.6374174, 0.30905056, 0.550437,
            0.1170066, 0.59754527, 0.7815346,
        ];

        for value in values {
            assert_eq!(rand.next_f32(), value);
        }
    }

    #[test]
    fn test_next_i64() {
        let mut rand = LegacyRand::from_seed(0);

        let values: [i64; 10] = [
            -4962768465676381896,
            4437113781045784766,
            -6688467811848818630,
            -8292973307042192125,
            -7423979211207825555,
            6146794652083548235,
            7105486291024734541,
            -279624296851435688,
            -2228689144322150137,
            -1083761183081836303,
        ];

        for value in values {
            assert_eq!(rand.next_i64(), value);
        }
    }

    #[test]
    fn test_next_bool() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [
            true, true, false, true, true, false, true, false, true, true,
        ];

        for value in values {
            assert_eq!(rand.next_bool(), value);
        }
    }

    #[test]
    fn test_next_gaussian() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [
            0.8025330637390305,
            -0.9015460884175122,
            2.080920790428163,
            0.7637707684364894,
            0.9845745328825128,
            -1.6834122587673428,
            -0.027290262907887285,
            0.11524570286202315,
            -0.39016704137993774,
            -0.643388813126449,
        ];

        for value in values {
            assert_eq!(rand.next_gaussian(), value);
        }
    }

    #[test]
    fn test_next_triangular() {
        let mut rand = LegacyRand::from_seed(0);

        let values = [
            124.52156858525856,
            104.34902101162372,
            113.2163439160276,
            70.01738222704547,
            96.89666691951828,
            107.30284075808541,
            106.16817675813144,
            79.11264482608078,
            73.96721613927062,
            81.72419521080646,
        ];

        for value in values {
            assert_eq!(rand.next_triangular(100f64, 50f64), value);
        }
    }

    #[test]
    fn test_split() {
        let mut original_rand = LegacyRand::from_seed(0);
        assert_eq!(original_rand.next_i64(), -4962768465676381896i64);

        let mut original_rand = LegacyRand::from_seed(0);
        {
            let splitter: LegacySplitter = original_rand.next_splitter();
            assert_eq!(splitter.seed, (-4962768465676381896i64) as u64);

            let mut rand = splitter.split_string("minecraft:offset");
            assert_eq!(rand.next_i32(), 103436829);
        }

        let mut original_rand = LegacyRand::from_seed(0);
        let mut new_rand = original_rand.split();
        {
            let splitter = new_rand.next_splitter();

            let mut rand1 = splitter.split_string("TEST STRING");
            assert_eq!(rand1.next_i32(), -1170413697);

            let mut rand2 = splitter.split_u64(10);
            assert_eq!(rand2.next_i32(), -1157793070);

            let mut rand3 = splitter.split_pos(1, 11, -111);
            assert_eq!(rand3.next_i32(), -1213890343);
        }

        assert_eq!(original_rand.next_i32(), 1033096058);
        assert_eq!(new_rand.next_i32(), -888301832);
    }
}
