use super::{gaussian::GaussianGenerator, Random, RandomSplitter};

pub struct Xoroshiro {
    seed: XoroshiroSeed,
    internal_next_gaussian: f64,
    internal_has_next_gaussian: bool,
}

impl Xoroshiro {
    fn new(lo: i64, hi: i64) -> Self {
        if (lo | hi) == 0 {
            return Self {
                seed: XoroshiroSeed {
                    lo: -7046029254386353131i64,
                    hi: 7640891576956012809i64,
                },
                internal_next_gaussian: 0f64,
                internal_has_next_gaussian: false,
            };
        }
        Self {
            seed: XoroshiroSeed { lo, hi },
            internal_next_gaussian: 0f64,
            internal_has_next_gaussian: false,
        }
    }

    fn next(&mut self, bits: u64) -> u64 {
        (self.seed.next() as u64) >> (64 - bits)
    }
}

impl GaussianGenerator for Xoroshiro {
    fn stored_next_gaussian(&self) -> f64 {
        self.internal_next_gaussian
    }

    fn has_next_gaussian(&self) -> bool {
        self.internal_has_next_gaussian
    }

    fn set_stored_next_gaussian(&mut self, value: f64) {
        self.internal_next_gaussian = value;
    }

    fn set_has_next_gaussian(&mut self, value: bool) {
        self.internal_has_next_gaussian = value;
    }
}

impl Random for Xoroshiro {
    fn split(&mut self) -> Self {
        Self::new(self.seed.next(), self.seed.next())
    }

    fn next_splitter(&mut self) -> impl RandomSplitter {
        XoroshiroSplitter::new(self.seed.next(), self.seed.next())
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = XoroshiroSeed::mixed_seed(seed);
        self.set_has_next_gaussian(false);
    }

    fn next_i32(&mut self) -> i32 {
        self.seed.next() as i32
    }

    fn next_bounded_i32(&mut self, bound: i32) -> i32 {
        let mut l = (self.next_i32() as u64) & 0xFFFFFFFF;
        let mut m = l.wrapping_mul(bound as u64);
        let mut n = m & 4294967295u64;
        if n < bound as u64 {
            let i = (((!bound).wrapping_add(1)) as u64) % (bound as u64);
            while n < i {
                l = self.next_i32() as u64;
                m = l.wrapping_mul(bound as u64);
                n = m & 4294967295u64;
            }
        }
        let o = m >> 32;
        o as i32
    }

    fn next_i64(&mut self) -> i64 {
        self.seed.next()
    }

    fn next_bool(&mut self) -> bool {
        (self.seed.next() & 1) != 0
    }

    fn next_f32(&mut self) -> f32 {
        self.next(24) as f32 * 5.9604645E-8f32
    }

    fn next_f64(&mut self) -> f64 {
        self.next(53) as f64 * 1.110223E-16f32 as f64
    }

    fn next_gaussian(&mut self) -> f64 {
        self.calculate_gaussian()
    }

    fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.seed.next();
        }
    }
}

struct XoroshiroSeed {
    lo: i64,
    hi: i64,
}

fn mix_stafford_13(seed: i64) -> i64 {
    let seed = (seed ^ ((seed as u64) >> 30) as i64).wrapping_mul(-4658895280553007687i64);
    let seed = (seed ^ ((seed as u64) >> 27) as i64).wrapping_mul(-7723592293110705685i64);
    seed ^ ((seed as u64) >> 31) as i64
}

impl XoroshiroSeed {
    #[allow(dead_code)]
    fn split(&self, lo: i64, hi: i64) -> Self {
        Self {
            lo: self.lo ^ lo,
            hi: self.hi ^ hi,
        }
    }

    fn mix(&self) -> Self {
        Self {
            lo: mix_stafford_13(self.lo),
            hi: mix_stafford_13(self.hi),
        }
    }

    fn next(&mut self) -> i64 {
        let l = self.lo;
        let m = self.hi;
        let n = l.wrapping_add(m).rotate_left(17).wrapping_add(l);
        let o = m ^ l;
        self.lo = l.rotate_left(49) ^ o ^ (o << 21);
        self.hi = o.rotate_left(28);
        n
    }

    fn unmixed_seed(seed: i64) -> Self {
        let l = seed ^ 7640891576956012809i64;
        let m = l + -7046029254386353131i64;
        Self { lo: l, hi: m }
    }

    fn mixed_seed(seed: i64) -> Self {
        Self::unmixed_seed(seed).mix()
    }
}

pub struct XoroshiroSplitter {
    lo: i64,
    hi: i64,
}

impl XoroshiroSplitter {
    pub fn new(lo: i64, hi: i64) -> Self {
        Self { lo, hi }
    }
}

fn hash_pos(x: i32, y: i32, z: i32) -> i64 {
    let l =
        ((x.wrapping_mul(3129871)) as i64) ^ ((z as i64).wrapping_mul(116129781i64)) ^ (y as i64);
    let l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861i64)
        .wrapping_add(l.wrapping_mul(11i64));
    l >> 16
}

impl RandomSplitter for XoroshiroSplitter {
    fn split_pos(&self, x: i32, y: i32, z: i32) -> impl Random {
        let l = hash_pos(x, y, z);
        let m = l ^ self.lo;
        Xoroshiro::new(m, self.hi)
    }

    fn split_i64(&self, seed: i64) -> impl Random {
        Xoroshiro::new(seed ^ self.lo, seed ^ self.hi)
    }

    fn split_string(&self, seed: &str) -> impl Random {
        let bytes = md5::compute(seed.as_bytes());
        let l = i64::from_be_bytes(bytes[0..8].try_into().expect("incorrect length"));
        let m = i64::from_be_bytes(bytes[8..16].try_into().expect("incorrect length"));

        let new_seed = XoroshiroSeed { lo: l, hi: m }.split(self.lo, self.hi);
        Xoroshiro::new(new_seed.lo, new_seed.hi)
    }
}

#[cfg(test)]
mod tests {
    use crate::random::{Random, RandomSplitter};

    use super::{hash_pos, mix_stafford_13, Xoroshiro, XoroshiroSeed};

    // Values checked against results from the equivalent Java source

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
            assert_eq!(hash_pos(x, y, z), value);
        }
    }

    #[test]
    fn test_mix_stafford_13() {
        let values: [(i64, i64); 31] = [
            (0, 0),
            (1, 6238072747940578789),
            (64, -8456553050427055661),
            (4096, -1125827887270283392),
            (262144, -120227641678947436),
            (16777216, 6406066033425044679),
            (1073741824, 3143522559155490559),
            (16, -2773008118984693571),
            (1024, 8101005175654470197),
            (65536, -3551754741763842827),
            (4194304, -2737109459693184599),
            (2, -2606959012126976886),
            (128, -5825874238589581082),
            (8192, 1111983794319025228),
            (524288, -7964047577924347155),
            (33554432, -5634612006859462257),
            (2147483648, -1436547171018572641),
            (137438953472, -4514638798598940860),
            (8796093022208, -610572083552328405),
            (562949953421312, -263574021372026223),
            (36028797018963968, 7868130499179604987),
            (253, -4045451768301188906),
            (127, -6873224393826578139),
            (8447, 6670985465942597767),
            (524543, -6228499289678716485),
            (33554687, 2630391896919662492),
            (2147483903, -6879633228472053040),
            (137438953727, -5817997684975131823),
            (8796093022463, 2384436581894988729),
            (562949953421567, -5076179956679497213),
            (36028797018964223, -5993365784811617721),
        ];

        for (input, output) in values {
            assert_eq!(mix_stafford_13(input), output);
        }
    }

    #[test]
    fn test_next_i32() {
        let values = [
            -160476802,
            781697906,
            653572596,
            1337520923,
            -505875771,
            -47281585,
            342195906,
            1417498593,
            -1478887443,
            1560080270,
        ];

        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        for value in values {
            assert_eq!(xoroshiro.next_i32(), value);
        }
    }

    #[test]
    fn test_next_bounded_i32() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values = [9, 1, 1, 3, 8, 9, 0, 3, 6, 3];
        for value in values {
            assert_eq!(xoroshiro.next_bounded_i32(10), value);
        }

        let values = [
            9784805, 470346, 13560642, 7320226, 14949645, 13460529, 2824352, 10938308, 14146127,
            4549185,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_bounded_i32(0xFFFFFF), value);
        }
    }

    #[test]
    fn test_next_between_i32() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values = [99, 59, 57, 65, 94, 100, 54, 66, 83, 68];
        for value in values {
            assert_eq!(xoroshiro.next_inbetween_i32(50, 100), value);
        }
    }

    #[test]
    fn test_next_inbetween_exclusive() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values = [98, 59, 57, 65, 94, 99, 53, 66, 82, 68];
        for value in values {
            assert_eq!(xoroshiro.next_inbetween_i32_exclusive(50, 100), value);
        }
    }

    #[test]
    fn test_next_f64() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [f64; 10] = [
            0.16474369376959186,
            0.7997457290026366,
            0.2511961888876212,
            0.11712489470639631,
            0.0997124786680137,
            0.7566797430601416,
            0.7723285712021574,
            0.9420469457586381,
            0.48056202536813664,
            0.6099690583914598,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_f64(), value);
        }
    }

    #[test]
    fn test_next_f32() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [f32; 10] = [
            0.16474366,
            0.7997457,
            0.25119615,
            0.117124856,
            0.09971243,
            0.7566797,
            0.77232856,
            0.94204694,
            0.48056197,
            0.609969,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_f32(), value);
        }
    }

    #[test]
    fn test_next_i64() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [i64; 10] = [
            3038984756725240190,
            -3694039286755638414,
            4633751808701151732,
            2160572957309072155,
            1839370574944072389,
            -4488466507718817201,
            -4199796579929588030,
            -1069045159880208415,
            8864804693509535725,
            -7194800960680693874,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_i64(), value);
        }
    }

    #[test]
    fn test_next_bool() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [bool; 10] = [
            false, false, false, true, true, true, false, true, true, false,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_bool(), value);
        }
    }

    #[test]
    fn test_next_gaussian() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [f64; 10] = [
            -0.48540690699780015,
            0.43399227545320296,
            -0.3283265251019599,
            -0.5052497078202575,
            -0.3772512828630807,
            0.2419080215945433,
            -0.42622066207565135,
            2.411315261138953,
            -1.1419147030553274,
            -0.05849758093810378,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_gaussian(), value);
        }
    }

    #[test]
    fn test_next_triangular() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let values: [f64; 10] = [
            6.824989823834776,
            10.670356470906125,
            6.71516367803936,
            9.151408127217596,
            9.352964834883384,
            8.291618967842293,
            8.954549938640508,
            11.833001837470519,
            10.65851306020791,
            11.684676364031647,
        ];
        for value in values {
            assert_eq!(xoroshiro.next_triangular(10f64, 5f64), value);
        }
    }

    #[test]
    fn test_split() {
        let seed = XoroshiroSeed::mixed_seed(0);
        let mut xoroshiro = Xoroshiro::new(seed.lo, seed.hi);

        let mut new_generator = xoroshiro.split();
        assert_eq!(new_generator.next_i32(), 542195535);

        let splitter = new_generator.next_splitter();
        let mut rand_1 = splitter.split_string("TEST STRING");
        assert_eq!(rand_1.next_i32(), -641435713);

        let mut rand_2 = splitter.split_i64(42069);
        assert_eq!(rand_2.next_i32(), -340700677);

        let mut rand_3 = splitter.split_pos(1337, 80085, -69420);
        assert_eq!(rand_3.next_i32(), 790449132);
    }
}
