use num_traits::Pow;
use pumpkin_core::random::{legacy_rand::LegacyRand, RandomImpl};

use super::{dot, GRADIENTS};

#[derive(Clone)]
pub struct SimplexNoiseSampler {
    permutation: Box<[u8]>,
    x_origin: f64,
    y_origin: f64,
    z_origin: f64,
}

impl SimplexNoiseSampler {
    const SQRT_3: f64 = 1.7320508075688772f64;
    const SKEW_FACTOR_2D: f64 = 0.5f64 * (Self::SQRT_3 - 1f64);
    const UNSKEW_FACTOR_2D: f64 = (3f64 - Self::SQRT_3) / 6f64;

    pub fn new(random: &mut impl RandomImpl) -> Self {
        let x_origin = random.next_f64() * 256f64;
        let y_origin = random.next_f64() * 256f64;
        let z_origin = random.next_f64() * 256f64;

        let mut permutation = [0u8; 256];

        permutation
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i as u8);

        for i in 0..256 {
            let j = random.next_bounded_i32(256 - i) as usize;
            permutation.swap(i as usize, i as usize + j);
        }

        Self {
            permutation: Box::new(permutation),
            x_origin,
            y_origin,
            z_origin,
        }
    }

    fn map(&self, input: i32) -> i32 {
        self.permutation[(input & 0xFF) as usize] as i32
    }

    fn grad(gradient_index: usize, x: f64, y: f64, z: f64, distance: f64) -> f64 {
        let d = z.mul_add(-z, y.mul_add(-y, x.mul_add(-x, distance)));
        if d < 0f64 {
            0f64
        } else {
            let d = d * d;
            d * d * dot(&GRADIENTS[gradient_index], x, y, z)
        }
    }

    pub fn sample_2d(&self, x: f64, y: f64) -> f64 {
        let d = (x + y) * Self::SKEW_FACTOR_2D;
        let i = (x + d).floor() as i32;
        let j = (y + d).floor() as i32;

        let e = (i.wrapping_add(j)) as f64 * Self::UNSKEW_FACTOR_2D;
        let f = i as f64 - e;
        let g = j as f64 - e;

        let h = x - f;
        let k = y - g;

        let (l, m) = if h > k { (1, 0) } else { (0, 1) };

        let n = h - l as f64 + Self::UNSKEW_FACTOR_2D;
        let o = k - m as f64 + Self::UNSKEW_FACTOR_2D;
        let p = 2f64.mul_add(Self::UNSKEW_FACTOR_2D, h - 1f64);
        let q = 2f64.mul_add(Self::UNSKEW_FACTOR_2D, k - 1f64);

        let r = i & 0xFF;
        let s = j & 0xFF;

        let t = self.map(r.wrapping_add(self.map(s))) % 12;
        let u = self.map(r.wrapping_add(l).wrapping_add(self.map(s.wrapping_add(m)))) % 12;
        let v = self.map(r.wrapping_add(1).wrapping_add(self.map(s.wrapping_add(1)))) % 12;

        let w = Self::grad(t as usize, h, k, 0f64, 0.5f64);
        let z = Self::grad(u as usize, n, o, 0f64, 0.5f64);
        let aa = Self::grad(v as usize, p, q, 0f64, 0.5f64);

        70f64 * (w + z + aa)
    }

    pub fn sample_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let e = (x + y + z) * 0.3333333333333333f64;

        let i = (x + e).floor() as i32;
        let j = (y + e).floor() as i32;
        let k = (z + e).floor() as i32;

        let g = (i.wrapping_add(j).wrapping_add(k)) as f64 * 0.16666666666666666f64;
        let h = i as f64 - g;
        let l = j as f64 - g;
        let m = k as f64 - g;

        let n = x - h;
        let o = y - l;
        let p = z - m;

        let (q, r, s, t, u, v) = if n >= o {
            if o >= p {
                (1, 0, 0, 1, 1, 0)
            } else if n >= p {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if o < p {
            (0, 0, 1, 0, 1, 1)
        } else if n < p {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let w = n - q as f64 + 0.16666666666666666f64;
        let aa = o - r as f64 + 0.16666666666666666f64;
        let ab = p - s as f64 + 0.16666666666666666f64;

        let ac = n - t as f64 + 0.3333333333333333f64;
        let ad = o - u as f64 + 0.3333333333333333f64;
        let ae = p - v as f64 + 0.3333333333333333f64;

        let af = n - 1f64 + 0.5f64;
        let ag = o - 1f64 + 0.5f64;
        let ah = p - 1f64 + 0.5f64;

        let ai = i & 0xFF;
        let aj = j & 0xFF;
        let ak = k & 0xFF;

        let al = self.map(ai.wrapping_add(self.map(aj.wrapping_add(self.map(ak))))) % 12;
        let am = self.map(
            ai.wrapping_add(q).wrapping_add(
                self.map(
                    aj.wrapping_add(r)
                        .wrapping_add(self.map(ak.wrapping_add(s))),
                ),
            ),
        ) % 12;
        let an = self.map(
            ai.wrapping_add(t).wrapping_add(
                self.map(
                    aj.wrapping_add(u)
                        .wrapping_add(self.map(ak.wrapping_add(v))),
                ),
            ),
        ) % 12;
        let ao = self.map(
            ai.wrapping_add(1).wrapping_add(
                self.map(
                    aj.wrapping_add(1)
                        .wrapping_add(self.map(ak.wrapping_add(1))),
                ),
            ),
        ) % 12;

        let ap = Self::grad(al as usize, n, o, p, 0.6f64);
        let aq = Self::grad(am as usize, w, aa, ab, 0.6f64);
        let ar = Self::grad(an as usize, ac, ad, ae, 0.6f64);
        let az = Self::grad(ao as usize, af, ag, ah, 0.6f64);

        32f64 * (ap + aq + ar + az)
    }
}

pub struct OctaveSimplexNoiseSampler {
    octave_samplers: Vec<Option<SimplexNoiseSampler>>,
    persistence: f64,
    lacunarity: f64,
}

impl OctaveSimplexNoiseSampler {
    pub fn new(random: &mut impl RandomImpl, octaves: &[i32]) -> Self {
        let mut octaves = Vec::from_iter(octaves);
        octaves.sort();

        let i = -**octaves.first().expect("Should have some octaves");
        let j = **octaves.last().expect("Should have some octaves");
        let k = i.wrapping_add(j).wrapping_add(1);

        let sampler = SimplexNoiseSampler::new(random);
        let l = j;
        let mut samplers: Vec<Option<SimplexNoiseSampler>> = Vec::with_capacity(k as usize);
        for _ in 0..k {
            samplers.push(None);
        }

        for m in (j + 1)..k {
            if m >= 0 && octaves.contains(&&(l - m)) {
                let sampler = SimplexNoiseSampler::new(random);
                samplers[m as usize] = Some(sampler);
            } else {
                random.skip(262);
            }
        }

        if j > 0 {
            let sample = sampler.sample_3d(sampler.x_origin, sampler.y_origin, sampler.z_origin);
            let n = (sample * 9.223372E18f32 as f64) as i64;
            let mut random = LegacyRand::from_seed(n as u64);

            for o in (0..=(l - 1)).rev() {
                if o < k && octaves.contains(&&(l - o)) {
                    let sampler = SimplexNoiseSampler::new(&mut random);
                    samplers[o as usize] = Some(sampler);
                } else {
                    random.skip(262);
                }
            }
        }

        if j >= 0 && j < k && octaves.contains(&&0) {
            samplers[j as usize] = Some(sampler);
        }

        Self {
            octave_samplers: samplers,
            persistence: 1f64 / (2f64.pow(k) - 1f64),
            lacunarity: 2f64.pow(j),
        }
    }

    pub fn sample(&self, x: f64, y: f64, use_origin: bool) -> f64 {
        let mut d = 0f64;
        let mut e = self.lacunarity;
        let mut f = self.persistence;

        for sampler in self.octave_samplers.iter() {
            if let Some(sampler) = sampler {
                d += sampler.sample_2d(
                    x.mul_add(e, if use_origin { sampler.x_origin } else { 0f64 }),
                    y.mul_add(e, if use_origin { sampler.y_origin } else { 0f64 }),
                ) * f;
            }

            e /= 2f64;
            f *= 2f64;
        }

        d
    }
}

#[cfg(test)]
mod octave_simplex_noise_sampler_test {
    use pumpkin_core::random::{xoroshiro128::Xoroshiro, RandomImpl};

    use crate::world_gen::noise::simplex::OctaveSimplexNoiseSampler;

    #[test]
    fn test_new() {
        let mut rand = Xoroshiro::from_seed(450);
        assert_eq!(rand.next_i32(), 1394613419);
        let sampler = OctaveSimplexNoiseSampler::new(&mut rand, &[-1, 1, 0]);

        assert_eq!(sampler.lacunarity, 2f64);
        assert_eq!(sampler.persistence, 0.14285714285714285);

        let values = [
            (33.48154133535127, 200.15584029786743, 239.82697852863149),
            (115.65071632913913, 5.88805286077266, 184.4887403898897),
            (64.69791492580848, 19.256055216755044, 97.01795462351956),
        ];

        assert_eq!(values.len(), sampler.octave_samplers.len());
        for (sampler, (x, y, z)) in sampler.octave_samplers.iter().zip(values) {
            match sampler {
                Some(sampler) => {
                    assert_eq!(sampler.x_origin, x);
                    assert_eq!(sampler.y_origin, y);
                    assert_eq!(sampler.z_origin, z);
                }
                None => panic!(),
            }
        }
    }

    #[test]
    fn test_sample() {
        let mut rand = Xoroshiro::from_seed(450);
        assert_eq!(rand.next_i32(), 1394613419);
        let sampler = OctaveSimplexNoiseSampler::new(&mut rand, &[-1, 1, 0]);

        let values_1 = [
            (
                (-1.3127900550351206E7, 792897.4979227383),
                -0.4321152413690901,
            ),
            (
                (-1.6920637874404985E7, -2.7155569346339065E8),
                -0.5262902093081003,
            ),
            (
                (4.3144247722741723E8, 5.681942883881191E8),
                0.11591369897395602,
            ),
            (
                (1.4302738270336467E8, -1.4548998886244193E8),
                -0.3879951077548365,
            ),
            (
                (-3.9028350711219925E8, -5.213995559811158E7),
                -0.7540785159288218,
            ),
            (
                (-1.3442750163759476E8, -6.725465365393716E8),
                0.31442035977402105,
            ),
            (
                (-1.1937282161424601E8, 3.2134650034986335E8),
                0.28218849676360336,
            ),
            (
                (-3.128475507865152E8, -3.014112871163455E8),
                0.593770404657594,
            ),
            (
                (1.2027011883589141E8, -5.045175636913682E8),
                -0.2893240282016911,
            ),
            (
                (-9.065155753781198E7, 6106991.342893547),
                -0.3402301205344082,
            ),
        ];

        for ((x, y), sample) in values_1 {
            assert_eq!(sampler.sample(x, y, false), sample);
        }

        let values_2 = [
            (
                (-1.3127900550351206E7, 792897.4979227383),
                0.21834818545873672,
            ),
            (
                (-1.6920637874404985E7, -2.7155569346339065E8),
                0.025042742676442978,
            ),
            (
                (4.3144247722741723E8, 5.681942883881191E8),
                0.3738693783591451,
            ),
            (
                (1.4302738270336467E8, -1.4548998886244193E8),
                -0.023113657524218345,
            ),
            (
                (-3.9028350711219925E8, -5.213995559811158E7),
                0.5195582376240916,
            ),
            (
                (-1.3442750163759476E8, -6.725465365393716E8),
                0.020366186088347903,
            ),
            (
                (-1.1937282161424601E8, 3.2134650034986335E8),
                -0.10921072611129382,
            ),
            (
                (-3.128475507865152E8, -3.014112871163455E8),
                0.18066933648141983,
            ),
            (
                (1.2027011883589141E8, -5.045175636913682E8),
                -0.36788084946294336,
            ),
            (
                (-9.065155753781198E7, 6106991.342893547),
                -0.5677921377363926,
            ),
        ];

        for ((x, y), sample) in values_2 {
            assert_eq!(sampler.sample(x, y, true), sample);
        }
    }
}
#[cfg(test)]
mod simplex_noise_sampler_test {
    use std::ops::Deref;

    use pumpkin_core::random::{xoroshiro128::Xoroshiro, RandomImpl};

    use crate::world_gen::noise::simplex::SimplexNoiseSampler;

    #[test]
    fn test_create() {
        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = SimplexNoiseSampler::new(&mut rand);
        assert_eq!(sampler.x_origin, 48.58072036717974f64);
        assert_eq!(sampler.y_origin, 110.73235882678037f64);
        assert_eq!(sampler.z_origin, 65.26438852860176f64);

        let permutation: [u8; 256] = [
            159, 113, 41, 143, 203, 123, 95, 177, 25, 79, 229, 219, 194, 60, 130, 14, 83, 99, 24,
            202, 207, 232, 167, 152, 220, 201, 29, 235, 87, 147, 74, 160, 155, 97, 111, 31, 85,
            205, 115, 50, 13, 171, 77, 237, 149, 116, 209, 174, 169, 109, 221, 9, 166, 84, 54, 216,
            121, 106, 211, 16, 69, 244, 65, 192, 183, 146, 124, 37, 56, 45, 193, 158, 126, 217, 36,
            255, 162, 163, 230, 103, 63, 90, 191, 214, 20, 138, 32, 39, 238, 67, 64, 105, 250, 140,
            148, 114, 68, 75, 200, 161, 239, 125, 227, 199, 101, 61, 175, 107, 129, 240, 170, 51,
            139, 86, 186, 145, 212, 178, 30, 251, 89, 226, 120, 153, 47, 141, 233, 2, 179, 236, 1,
            19, 98, 21, 164, 108, 11, 23, 91, 204, 119, 88, 165, 195, 168, 26, 48, 206, 128, 6, 52,
            118, 110, 180, 197, 231, 117, 7, 3, 135, 224, 58, 82, 78, 4, 59, 222, 18, 72, 57, 150,
            43, 246, 100, 122, 112, 53, 133, 93, 17, 27, 210, 142, 234, 245, 80, 22, 46, 185, 172,
            71, 248, 33, 173, 76, 35, 40, 92, 228, 127, 254, 70, 42, 208, 73, 104, 187, 62, 154,
            243, 189, 241, 34, 66, 249, 94, 8, 12, 134, 132, 102, 242, 196, 218, 181, 28, 38, 15,
            151, 157, 247, 223, 198, 55, 188, 96, 0, 182, 49, 190, 156, 10, 215, 252, 131, 137,
            184, 176, 136, 81, 44, 213, 253, 144, 225, 5,
        ];
        assert_eq!(sampler.permutation.deref(), permutation);
    }

    #[test]
    fn test_sample_2d() {
        let data1 = [
            ((-50000, 0), -0.013008608535752102),
            ((-49999, 1000), 0.0),
            ((-49998, 2000), -0.03787856584046271),
            ((-49997, 3000), 0.0),
            ((-49996, 4000), 0.5015373706471664),
            ((-49995, 5000), -0.032797908620906514),
            ((-49994, 6000), -0.19158655563621785),
            ((-49993, 7000), 0.49893473629544977),
            ((-49992, 8000), 0.31585737840402556),
            ((-49991, 9000), 0.43909577227435836),
        ];

        let data2 = [
            (
                (-3.134738528791615E8, 5.676610095659718E7),
                0.018940199193618792,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8),
                -0.1417598930091471,
            ),
            (
                (6.439373693833767E8, -3.36218773041759E8),
                0.07129176668335062,
            ),
            (
                (1.353820060118252E8, -3.204701624793043E8),
                0.330648835988156,
            ),
            (
                (-6906850.625560562, 1.0153663948838013E8),
                0.46826928755778685,
            ),
            (
                (-7.108376621385525E7, -2.029413580824217E8),
                -0.515950097501492,
            ),
            (
                (1.0591429119126628E8, -4.7911044364543396E8),
                -0.5467822192664874,
            ),
            (
                (4.04615501401398E7, -3.074409286586152E8),
                0.7470460844090322,
            ),
            (
                (-4.8645283544246924E8, -3.922570151180015E8),
                0.8521699147242563,
            ),
            (
                (2.861710031285905E8, -1.8973201372718483E8),
                0.1889297962671115,
            ),
            (
                (2.885407603819252E8, -3.358708100884505E7),
                0.24006029504945695,
            ),
            (
                (3.6548491156354237E8, 7.995429702025633E7),
                -0.8114171447379924,
            ),
            (
                (1.3298684552869435E8, 3.6743804723880893E8),
                0.07042306408164949,
            ),
            (
                (-1.3123184148036437E8, -2.722300890805201E8),
                0.5093850689193259,
            ),
            (
                (-5.56047682304707E8, 3.554803693060646E8),
                -0.6343788467687929,
            ),
            (
                (5.638216625134594E8, -2.236907346192737E8),
                0.5848746152449286,
            ),
            (
                (-5.436956979127073E7, -1.129261611506945E8),
                -0.05456282199582522,
            ),
            (
                (1.0915760091641709E8, 1.932642099859593E7),
                -0.273739377096594,
            ),
            (
                (-6.73911758014991E8, -2.2147483413687566E8),
                0.05464681163741797,
            ),
            (
                (-2.4827386778136212E8, -2.6640208832089204E8),
                -0.0902449424742273,
            ),
        ];

        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = SimplexNoiseSampler::new(&mut rand);
        for ((x, y), sample) in data1 {
            assert_eq!(sampler.sample_2d(x as f64, y as f64), sample);
        }

        for ((x, y), sample) in data2 {
            assert_eq!(sampler.sample_2d(x, y), sample);
        }
    }

    #[test]
    fn test_sample_3d() {
        let data = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                ),
                -0.07626353895981935,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8, 6.797037355570006E8),
                0.0,
            ),
            (
                (
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                ),
                -0.5919400355725402,
            ),
            (
                (
                    1.353820060118252E8,
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                ),
                -0.5220477236433517,
            ),
            (
                (
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                -0.39146687767898636,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                ),
                -0.629386846329711,
            ),
            (
                (
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                    -2918719.2277242197,
                ),
                0.5427502531663232,
            ),
            (
                (
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                ),
                -0.4273080639878097,
            ),
            (
                (
                    -4.8645283544246924E8,
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                ),
                0.32129944093252394,
            ),
            (
                (
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                0.35839032946039706,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                ),
                -0.02451312935907038,
            ),
            (
                (
                    3.6548491156354237E8,
                    7.995429702025633E7,
                    2.509991661702412E8,
                ),
                -0.36830526266318003,
            ),
            (
                (
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                ),
                -0.023683302916542803,
            ),
            (
                (
                    -1.3123184148036437E8,
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                ),
                -0.261629562325043,
            ),
            (
                (
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.4959372930161496,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                ),
                -0.06079315675880484,
            ),
            (
                (
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                    -1.7909512156895646E8,
                ),
                -0.37726907424345196,
            ),
            (
                (
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                ),
                0.37747828159811136,
            ),
            (
                (
                    -6.73911758014991E8,
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                ),
                -0.32929020207000603,
            ),
            (
                (
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                -0.3046390200444667,
            ),
        ];

        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = SimplexNoiseSampler::new(&mut rand);
        for ((x, y, z), sample) in data {
            assert_eq!(sampler.sample_3d(x, y, z), sample);
        }
    }
}
