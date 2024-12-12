use std::sync::Arc;

use num_traits::Pow;
use pumpkin_core::random::RandomGenerator;

use super::{lerp3, GRADIENTS};

pub struct PerlinNoiseSampler {
    permutation: [u8; 256],
    x_origin: f64,
    y_origin: f64,
    z_origin: f64,
}

impl PerlinNoiseSampler {
    pub fn new(random: &mut RandomGenerator) -> Self {
        let x_origin = random.next_f64() * 256f64;
        let y_origin = random.next_f64() * 256f64;
        let z_origin = random.next_f64() * 256f64;

        let mut permutation = [0u8; 256];

        permutation
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i as u8);

        for i in 0..256 {
            let j = random.next_bounded_i32((256 - i) as i32) as usize;
            permutation.swap(i, i + j);
        }

        Self {
            permutation,
            x_origin,
            y_origin,
            z_origin,
        }
    }

    #[inline]
    pub fn sample_flat_y(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample_no_fade(x, y, z, 0f64, 0f64)
    }

    pub fn sample_no_fade(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let true_x = x + self.x_origin;
        let true_y = y + self.y_origin;
        let true_z = z + self.z_origin;

        let x_floor = true_x.floor();
        let y_floor = true_y.floor();
        let z_floor = true_z.floor();

        let x_dec = true_x - x_floor;
        let y_dec = true_y - y_floor;
        let z_dec = true_z - z_floor;

        let y_noise = if y_scale != 0f64 {
            let raw_y_dec = if y_max >= 0f64 && y_max < y_dec {
                y_max
            } else {
                y_dec
            };
            (raw_y_dec / y_scale + 1E-7f32 as f64).floor() * y_scale
        } else {
            0f64
        };

        self.sample(
            x_floor as i32,
            y_floor as i32,
            z_floor as i32,
            x_dec,
            y_dec - y_noise,
            z_dec,
            y_dec,
        )
    }

    #[inline]
    fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
        GRADIENTS[(hash & 15) as usize].dot(x, y, z)
    }

    #[inline]
    fn perlin_fade(value: f64) -> f64 {
        value * value * value * (value * (value * 6f64 - 15f64) + 10f64)
    }

    #[inline]
    fn map(&self, input: i32) -> i32 {
        self.permutation[(input & 0xFF) as usize] as i32
    }

    #[allow(clippy::too_many_arguments)]
    fn sample(
        &self,
        x: i32,
        y: i32,
        z: i32,
        local_x: f64,
        local_y: f64,
        local_z: f64,
        fade_local_y: f64,
    ) -> f64 {
        let i = self.map(x);
        let j = self.map(x + 1);
        let k = self.map(i + y);
        let l = self.map(i + y + 1);

        let m = self.map(j + y);
        let n = self.map(j + y + 1);

        let d = Self::grad(self.map(k + z), local_x, local_y, local_z);
        let e = Self::grad(self.map(m + z), local_x - 1f64, local_y, local_z);
        let f = Self::grad(self.map(l + z), local_x, local_y - 1f64, local_z);
        let g = Self::grad(self.map(n + z), local_x - 1f64, local_y - 1f64, local_z);
        let h = Self::grad(self.map(k + z + 1), local_x, local_y, local_z - 1f64);
        let o = Self::grad(self.map(m + z + 1), local_x - 1f64, local_y, local_z - 1f64);
        let p = Self::grad(self.map(l + z + 1), local_x, local_y - 1f64, local_z - 1f64);
        let q = Self::grad(
            self.map(n + z + 1),
            local_x - 1f64,
            local_y - 1f64,
            local_z - 1f64,
        );
        let r = Self::perlin_fade(local_x);
        let s = Self::perlin_fade(fade_local_y);
        let t = Self::perlin_fade(local_z);

        lerp3(r, s, t, d, e, f, g, h, o, p, q)
    }
}

pub struct OctavePerlinNoiseSampler {
    octave_samplers: Box<[Option<PerlinNoiseSampler>]>,
    amplitudes: Box<[f64]>,
    first_octave: i32,
    persistences: Box<[f64]>,
    lacunarities: Box<[f64]>,
    max_value: f64,
}

impl OctavePerlinNoiseSampler {
    pub(crate) fn get_octave(&self, octave: i32) -> Option<&PerlinNoiseSampler> {
        match self
            .octave_samplers
            .get(self.octave_samplers.len() - 1 - octave as usize)
        {
            Some(octave) => octave.as_ref(),
            None => None,
        }
    }

    pub(crate) fn get_total_amplitude(scale: f64, persistence: f64, amplitudes: &[f64]) -> f64 {
        let mut d = 0f64;
        let mut e = persistence;

        for amplitude in amplitudes.iter() {
            d += amplitude * scale * e;
            e /= 2f64;
        }

        d
    }

    #[inline]
    pub fn maintain_precision(value: f64) -> f64 {
        value - (value / 3.3554432E7f64 + 0.5f64).floor() * 3.3554432E7f64
    }

    pub fn calculate_amplitudes(octaves: &[i32]) -> (i32, Vec<f64>) {
        let mut octaves = Vec::from_iter(octaves);
        octaves.sort();

        let i = -**octaves.first().expect("we should have some octaves");
        let j = **octaves.last().expect("we should have some octaves");
        let k = i + j + 1;

        let mut double_list: Vec<f64> = Vec::with_capacity(k as usize);
        for _ in 0..k {
            double_list.push(0f64)
        }

        for l in octaves {
            double_list[(l + i) as usize] = 1f64;
        }

        (-i, double_list)
    }

    pub fn new(
        random: &mut RandomGenerator,
        first_octave: i32,
        amplitudes: &[f64],
        legacy: bool,
    ) -> Self {
        let i = amplitudes.len();
        let j = -first_octave;

        let mut samplers: Vec<Option<PerlinNoiseSampler>> = Vec::with_capacity(i);
        for _ in 0..i {
            samplers.push(None);
        }

        if legacy {
            let sampler = PerlinNoiseSampler::new(random);
            if j >= 0 && j < i as i32 {
                let d = amplitudes[j as usize];
                if d != 0f64 {
                    samplers[j as usize] = Some(sampler);
                }
            }

            for kx in (0..j as usize).rev() {
                if kx < i {
                    let e = amplitudes[kx];
                    if e != 0f64 {
                        samplers[kx] = Some(PerlinNoiseSampler::new(random));
                    } else {
                        random.skip(262);
                    }
                } else {
                    random.skip(262);
                }
            }
        } else {
            let splitter = random.next_splitter();
            for k in 0..i {
                if amplitudes[k] != 0f64 {
                    let l = first_octave + k as i32;
                    samplers[k] = Some(PerlinNoiseSampler::new(
                        &mut splitter.split_string(&format!("octave_{}", l)),
                    ));
                }
            }
        }

        let mut persistence =
            2f64.pow((i as i32).wrapping_sub(1) as f64) / (2f64.pow(i as f64) - 1f64);
        let mut lacunarity = 2f64.pow((-j) as f64);
        let max_value = Self::get_total_amplitude(2f64, persistence, amplitudes);

        let persistences: Vec<f64> = (0..amplitudes.len())
            .map(|_| {
                let result = persistence;
                persistence /= 2f64;
                result
            })
            .collect();
        let lacunarities: Vec<f64> = (0..amplitudes.len())
            .map(|_| {
                let result = lacunarity;
                lacunarity *= 2f64;
                result
            })
            .collect();

        Self {
            octave_samplers: samplers.into(),
            amplitudes: amplitudes.into(),
            first_octave,
            persistences: persistences.into(),
            lacunarities: lacunarities.into(),
            max_value,
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut d = 0f64;

        let num_octaves = self.octave_samplers.len();
        for i in 0..num_octaves {
            if let Some(sampler) = &self.octave_samplers[i] {
                let lacunarity = self.lacunarities[i];
                let amplitude = self.amplitudes[i];
                let persistence = self.persistences[i];

                let g = sampler.sample_no_fade(
                    Self::maintain_precision(x * lacunarity),
                    Self::maintain_precision(y * lacunarity),
                    Self::maintain_precision(z * lacunarity),
                    0f64,
                    0f64,
                );

                d += amplitude * g * persistence;
            }
        }

        d
    }
}

pub struct DoublePerlinNoiseParameters {
    first_octave: i32,
    amplitudes: &'static [f64],
    id: &'static str,
}

impl DoublePerlinNoiseParameters {
    pub(crate) const fn new(
        first_octave: i32,
        amplitudes: &'static [f64],
        id: &'static str,
    ) -> Self {
        Self {
            first_octave,
            amplitudes,
            id,
        }
    }

    pub fn id(&self) -> &'static str {
        self.id
    }
}

#[derive(Clone)]
pub struct DoublePerlinNoiseSampler {
    first_sampler: Arc<OctavePerlinNoiseSampler>,
    second_sampler: Arc<OctavePerlinNoiseSampler>,
    amplitude: f64,
    max_value: f64,
}

impl DoublePerlinNoiseSampler {
    fn create_amplitude(octaves: i32) -> f64 {
        0.1f64 * (1f64 + 1f64 / (octaves + 1) as f64)
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn new(
        rand: &mut RandomGenerator,
        parameters: &DoublePerlinNoiseParameters,
        legacy: bool,
    ) -> Self {
        let first_octave = parameters.first_octave;
        let amplitudes = parameters.amplitudes;

        let first_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);
        let second_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);

        let mut j = i32::MAX;
        let mut k = i32::MIN;

        for (index, amplitude) in amplitudes.iter().enumerate() {
            if *amplitude != 0f64 {
                j = i32::min(j, index as i32);
                k = i32::max(k, index as i32);
            }
        }

        let amplitude = 0.16666666666666666f64 / Self::create_amplitude(k - j);
        let max_value = (first_sampler.max_value + second_sampler.max_value) * amplitude;

        Self {
            first_sampler: first_sampler.into(),
            second_sampler: second_sampler.into(),
            amplitude,
            max_value,
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x * 1.0181268882175227f64;
        let e = y * 1.0181268882175227f64;
        let f = z * 1.0181268882175227f64;

        (self.first_sampler.sample(x, y, z) + self.second_sampler.sample(d, e, f)) * self.amplitude
    }
}

#[cfg(test)]
mod double_perlin_noise_sampler_test {
    use pumpkin_core::random::{
        legacy_rand::LegacyRand, xoroshiro128::Xoroshiro, RandomGenerator, RandomImpl,
    };

    use crate::world_gen::noise::perlin::{DoublePerlinNoiseParameters, DoublePerlinNoiseSampler};

    #[test]
    fn sample_legacy() {
        let mut rand = LegacyRand::from_seed(513513513);
        assert_eq!(rand.next_i32(), -1302745855);

        let mut rand_gen = RandomGenerator::Legacy(rand);
        let params = DoublePerlinNoiseParameters::new(0, &[4f64], "");
        let sampler = DoublePerlinNoiseSampler::new(&mut rand_gen, &params, true);

        let values = [
            (
                (
                    3.7329617139221236E7,
                    2.847228022372606E8,
                    -1.8244299064688918E8,
                ),
                -0.5044027150385925,
            ),
            (
                (
                    8.936597679535551E7,
                    1.491954533221004E8,
                    3.457494216166344E8,
                ),
                -1.0004671438756043,
            ),
            (
                (
                    -2.2479845046034336E8,
                    -4.085449163378981E7,
                    1.343082907470065E8,
                ),
                2.1781128778536973,
            ),
            (
                (
                    -1.9094944979652843E8,
                    3.695081561625232E8,
                    2.1566424798360935E8,
                ),
                -1.2571847948126453,
            ),
            (
                (
                    1.8486356004931596E8,
                    -4.148713734284534E8,
                    4.8687219454012525E8,
                ),
                -0.550285244015363,
            ),
            (
                (
                    1.7115351141710258E8,
                    -1.8835885697652313E8,
                    1.7031060329927653E8,
                ),
                -0.6953327750604766,
            ),
            (
                (
                    8.952317194270046E7,
                    -5.420942524023042E7,
                    -2.5987559023045145E7,
                ),
                2.7361630914824393,
            ),
            (
                (
                    -8.36195975247282E8,
                    -1.2167090318484206E8,
                    2.1237199673286602E8,
                ),
                -1.5518675789351004,
            ),
            (
                (
                    3.333103540906928E8,
                    5.088236187007203E8,
                    -3.521137809477999E8,
                ),
                0.6928720433082317,
            ),
            (
                (
                    7.82760234776598E7,
                    -2.5204361464037597E7,
                    -1.6615974590937865E8,
                ),
                -0.5102124930620466,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample)
        }
    }

    #[test]
    fn sample_xoroshiro() {
        let mut rand = Xoroshiro::from_seed(5);
        assert_eq!(rand.next_i32(), -1678727252);

        let mut rand_gen = RandomGenerator::Xoroshiro(rand);

        let params = DoublePerlinNoiseParameters::new(1, &[2f64, 4f64], "");

        let sampler = DoublePerlinNoiseSampler::new(&mut rand_gen, &params, false);

        let values = [
            (
                (
                    -2.4823401687190732E8,
                    1.6909869132832196E8,
                    1.0510057123823991E8,
                ),
                -0.09627881756376819,
            ),
            (
                (
                    1.2971355215791291E8,
                    -3.614855223614046E8,
                    1.9997149869463342E8,
                ),
                0.4412466810560897,
            ),
            (
                (
                    -1.9858224577678584E7,
                    2.5103843334053648E8,
                    2.253841390457064E8,
                ),
                -1.3086196098510068,
            ),
            (
                (
                    1.4243878295159304E8,
                    -1.9185612600051942E8,
                    4.7736284830701286E8,
                ),
                1.727683424808049,
            ),
            (
                (
                    -9.411241394159131E7,
                    4.4052130232611096E8,
                    5.1042225596740514E8,
                ),
                -0.4651812519989636,
            ),
            (
                (
                    3.007670445405074E8,
                    1.4630490674448165E8,
                    -1.681994537227527E8,
                ),
                -0.8607587886441551,
            ),
            (
                (
                    -2.290369962944646E8,
                    -4.9627750061129004E8,
                    9.751744069476394E7,
                ),
                -0.3592693708849225,
            ),
            (
                (
                    -5.380825223911383E7,
                    6.317706682942032E7,
                    -3.0105795661690116E8,
                ),
                0.7372424991843702,
            ),
            (
                (
                    -1.4261684559190175E8,
                    9.987839104129419E7,
                    3.3290027416415906E8,
                ),
                0.27706980571082485,
            ),
            (
                (
                    -8.881637146904664E7,
                    1.1033687270820947E8,
                    -1.0014482192140123E8,
                ),
                -0.4602443245357103,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample)
        }
    }
}

#[cfg(test)]
mod octave_perline_noise_sampler_test {
    use pumpkin_core::random::{
        legacy_rand::LegacyRand, xoroshiro128::Xoroshiro, RandomGenerator, RandomImpl,
    };

    use super::OctavePerlinNoiseSampler;

    #[test]
    fn test_create_xoroshiro() {
        let mut rand = Xoroshiro::from_seed(513513513);
        assert_eq!(rand.next_i32(), 404174895);

        let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[1, 2, 3]);
        assert_eq!(start, 1);
        assert_eq!(amplitudes, [1f64, 1f64, 1f64]);

        let mut rand_gen = RandomGenerator::Xoroshiro(rand);
        let sampler = OctavePerlinNoiseSampler::new(&mut rand_gen, start, &amplitudes, false);

        assert_eq!(sampler.first_octave, 1);
        assert_eq!(sampler.persistences[0], 0.5714285714285714f64);
        assert_eq!(sampler.lacunarities[0], 2f64);
        assert_eq!(sampler.max_value, 2f64);

        let coords = [
            (210.19539348148294, 203.08258445596215, 45.29925114984684),
            (24.841250686920773, 181.62678157390076, 69.49871248131629),
            (21.65886467061867, 97.80131502331685, 225.9273676334467),
        ];

        for (sampler, (x, y, z)) in sampler.octave_samplers.iter().zip(coords) {
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
    fn test_create_legacy() {
        let mut rand = LegacyRand::from_seed(513513513);
        assert_eq!(rand.next_i32(), -1302745855);

        let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[0]);
        assert_eq!(start, 0);
        assert_eq!(amplitudes, [1f64]);

        let mut rand_gen = RandomGenerator::Legacy(rand);
        let sampler = OctavePerlinNoiseSampler::new(&mut rand_gen, start, &amplitudes, true);
        assert_eq!(sampler.first_octave, 0);
        assert_eq!(sampler.persistences[0], 1f64);
        assert_eq!(sampler.lacunarities[0], 1f64);
        assert_eq!(sampler.max_value, 2f64);

        let coords = [(226.220117499588, 32.67924779023767, 202.84067325597647)];

        for (sampler, (x, y, z)) in sampler.octave_samplers.iter().zip(coords) {
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
        let mut rand = Xoroshiro::from_seed(513513513);
        assert_eq!(rand.next_i32(), 404174895);

        let (start, amplitudes) = OctavePerlinNoiseSampler::calculate_amplitudes(&[1, 2, 3]);
        let mut rand_gen = RandomGenerator::Xoroshiro(rand);
        let sampler = OctavePerlinNoiseSampler::new(&mut rand_gen, start, &amplitudes, false);

        let values = [
            (
                (
                    1.4633897801218182E8,
                    3.360929121402108E8,
                    -1.7376184515043163E8,
                ),
                -0.16510137639683028,
            ),
            (
                (
                    -3.952093942501234E8,
                    -8.149682915016855E7,
                    2.0761709535397574E8,
                ),
                -0.19865227457826365,
            ),
            (
                (
                    1.0603518812861493E8,
                    -1.6028050039630303E8,
                    9.621510690305333E7,
                ),
                -0.16157548492944798,
            ),
            (
                (
                    -2.2789281609860754E8,
                    1.2416505757723756E8,
                    -3.047619296454517E8,
                ),
                -0.05762575118540847,
            ),
            (
                (
                    -1.6361322604690066E8,
                    -1.862652364900794E8,
                    9.03458926538596E7,
                ),
                0.21589404036742288,
            ),
            (
                (
                    -1.6074718857061076E8,
                    -4.816551924254624E8,
                    -9.930236785759543E7,
                ),
                0.1888188057014473,
            ),
            (
                (
                    -1.6848478115907547E8,
                    1.9495247771890038E8,
                    1.3780564333313772E8,
                ),
                0.23114508298896774,
            ),
            (
                (
                    2.5355640846261957E8,
                    -2.5973376726076955E8,
                    3.7834594620459855E7,
                ),
                -0.23703473310230702,
            ),
            (
                (
                    -8.636649828254433E7,
                    1.7017680431584623E8,
                    2.941033134334743E8,
                ),
                -0.14050102207739693,
            ),
            (
                (
                    -4.573784466442647E8,
                    1.789046617664721E8,
                    -5.515223967099891E8,
                ),
                -0.1422470544720957,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample(x, y, z), sample);
        }
    }
}

#[cfg(test)]
mod perlin_noise_sampler_test {
    use std::{fs, path::Path};

    use pumpkin_core::{
        assert_eq_delta,
        random::{xoroshiro128::Xoroshiro, RandomDeriverImpl, RandomGenerator, RandomImpl},
    };

    use crate::{read_data_from_file, world_gen::noise::perlin::PerlinNoiseSampler};

    use super::OctavePerlinNoiseSampler;

    #[test]
    fn test_create() {
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(111));
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = PerlinNoiseSampler::new(&mut rand);
        assert_eq!(sampler.x_origin, 48.58072036717974);
        assert_eq!(sampler.y_origin, 110.73235882678037);
        assert_eq!(sampler.z_origin, 65.26438852860176);

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
        assert_eq!(sampler.permutation, permutation);
    }

    #[test]
    fn test_no_y() {
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(111));
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = PerlinNoiseSampler::new(&mut rand);

        let values = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                ),
                0.38582139614602945,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8, 6.797037355570006E8),
                0.15777501333157193,
            ),
            (
                (
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                ),
                -0.2806135912409497,
            ),
            (
                (
                    1.353820060118252E8,
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                ),
                -0.15052865500837787,
            ),
            (
                (
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                -0.3079300694558318,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                ),
                0.03051312670440398,
            ),
            (
                (
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                    -2918719.2277242197,
                ),
                -0.11775123159138573,
            ),
            (
                (
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                ),
                0.08763639340713025,
            ),
            (
                (
                    -4.8645283544246924E8,
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                ),
                0.08857245482456311,
            ),
            (
                (
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                -0.2378339698793312,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                ),
                -0.46661747461279457,
            ),
            (
                (
                    3.6548491156354237E8,
                    7.995429702025633E7,
                    2.509991661702412E8,
                ),
                0.1671543972176835,
            ),
            (
                (
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                ),
                -0.2704070746642889,
            ),
            (
                (
                    -1.3123184148036437E8,
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                ),
                0.05049887915906969,
            ),
            (
                (
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.21178547899422662,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                ),
                0.03351245780858128,
            ),
            (
                (
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                    -1.7909512156895646E8,
                ),
                0.31670010349494726,
            ),
            (
                (
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                ),
                -0.13987439655026918,
            ),
            (
                (
                    -6.73911758014991E8,
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                ),
                0.07824440437151846,
            ),
            (
                (
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                -0.2989735599541437,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample_flat_y(x, y, z), sample);
        }
    }

    #[test]
    fn test_no_y_chunk() {
        let expected_data: Vec<(i32, i32, i32, f64)> =
            read_data_from_file!("../../../assets/perlin2_7_4.json");

        let mut rand = Xoroshiro::from_seed(0);
        let splitter = rand.next_splitter();
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));
        assert_eq!(rand.next_i32(), 1374487555);
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));

        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
        let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
        let sampler = sampler.get_octave(0).unwrap();

        assert_eq!(sampler.x_origin, 18.223354299069797);
        assert_eq!(sampler.y_origin, 93.99298907803595);
        assert_eq!(sampler.z_origin, 184.48198875745823);

        for (x, y, z, sample) in expected_data {
            let scale = 0.005;
            let result =
                sampler.sample_flat_y(x as f64 * scale, y as f64 * scale, z as f64 * scale);
            assert_eq_delta!(result, sample, f64::EPSILON);
        }
    }

    #[test]
    fn test_no_fade() {
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(111));
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = PerlinNoiseSampler::new(&mut rand);

        let values = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                    -1369026.560586418,
                    3.957311252810864E8,
                ),
                23234.47859421248,
            ),
            (
                (
                    6.797037355570006E8,
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                    1.353820060118252E8,
                ),
                -0.016403984198221984,
            ),
            (
                (
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                0.3444286491766397,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                ),
                0.03051312670440398,
            ),
            (
                (
                    -2918719.2277242197,
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                    -4.8645283544246924E8,
                ),
                0.3434020232968479,
            ),
            (
                (
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                -0.07935517045771859,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                    3.6548491156354237E8,
                    7.995429702025633E7,
                ),
                -0.46661747461279457,
            ),
            (
                (
                    2.509991661702412E8,
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                    -1.3123184148036437E8,
                ),
                0.0723439870279631,
            ),
            (
                (
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.656560662515624,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                ),
                0.03351245780858128,
            ),
            (
                (
                    -1.7909512156895646E8,
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                    -6.73911758014991E8,
                ),
                -0.2089142558681482,
            ),
            (
                (
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                0.38250837565598395,
            ),
            (
                (
                    3.618095500266467E8,
                    -1.785261966631494E8,
                    8.855575989580283E7,
                    -1.3702508894700047E8,
                    -3.564818414428105E8,
                ),
                0.00883370523171791,
            ),
            (
                (
                    3.585592594479808E7,
                    1.8822208340571395E8,
                    -386327.524558296,
                    -2.613548000006699E8,
                    1995562.4304017993,
                ),
                -0.27653878487738676,
            ),
            (
                (
                    3.0800276873619422E7,
                    1.166750302259058E7,
                    8.502636255675305E7,
                    4.347409652503064E8,
                    1.0678086363325526E8,
                ),
                -0.13800758751097497,
            ),
            (
                (
                    -2.797805968820768E8,
                    9.446376468140173E7,
                    2.2821543438325477E8,
                    -4.8176550369786626E8,
                    7.316871126959312E7,
                ),
                0.05505478945301634,
            ),
            (
                (
                    -2.236596113898912E7,
                    1.5296478602495643E8,
                    3.903966235164034E8,
                    9.40479475527148E7,
                    1.0948229366673347E8,
                ),
                0.1158678618158655,
            ),
            (
                (
                    3.5342596632385695E8,
                    3.1584773170834744E8,
                    -2.1860087172846535E8,
                    -1.8126626716239208E8,
                    -2.5263456116162892E7,
                ),
                -0.354953975313882,
            ),
            (
                (
                    -1.2711958434031656E8,
                    -4.541988855460623E7,
                    -1.375878074907788E8,
                    6.72693784001799E7,
                    6815739.665531283,
                ),
                -0.23849179316215247,
            ),
            (
                (
                    1.2660906027019228E8,
                    -3.3769609799741164E7,
                    -3.4331505330046E8,
                    -6.663866659430536E7,
                    -1.6603843763414428E8,
                ),
                0.07974650858448407,
            ),
        ];

        for ((x, y, z, y_scale, y_max), sample) in values {
            assert_eq!(sampler.sample_no_fade(x, y, z, y_scale, y_max), sample);
        }
    }

    #[test]
    fn test_no_fade_chunk() {
        let expected_data: Vec<(i32, i32, i32, f64)> =
            read_data_from_file!("../../../assets/perlin_7_4.json");

        let mut rand = Xoroshiro::from_seed(0);
        let splitter = rand.next_splitter();
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));
        assert_eq!(rand.next_i32(), 1374487555);
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));

        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
        let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
        let sampler = sampler.get_octave(0).unwrap();

        assert_eq!(sampler.x_origin, 18.223354299069797);
        assert_eq!(sampler.y_origin, 93.99298907803595);
        assert_eq!(sampler.z_origin, 184.48198875745823);

        for (x, y, z, sample) in expected_data {
            let scale = 0.005;
            let max_y = scale * 2f64;
            let result = sampler.sample_no_fade(
                x as f64 * scale,
                y as f64 * scale,
                z as f64 * scale,
                scale,
                max_y,
            );
            assert_eq_delta!(result, sample, f64::EPSILON);
        }
    }

    #[test]
    fn test_precision() {
        let values = [
            2.5E-4,
            1.25E-4,
            6.25E-5,
            3.125E-5,
            1.5625E-5,
            7.8125E-6,
            3.90625E-6,
            1.953125E-6,
            9.765625E-7,
            4.8828125E-7,
            2.44140625E-7,
            1.220703125E-7,
            6.103515625E-8,
            3.0517578125E-8,
            1.52587890625E-8,
            7.62939453125E-9,
            3.814697265625E-9,
            1.9073486328125E-9,
            9.5367431640625E-10,
            4.76837158203125E-10,
            2.384185791015625E-10,
            1.1920928955078125E-10,
            5.960464477539063E-11,
            2.980232238769531E-11,
            1.4901161193847657E-11,
            7.450580596923828E-12,
            3.725290298461914E-12,
            1.862645149230957E-12,
            9.313225746154785E-13,
        ];
        let mut value_iter = values.iter();

        for x in 1..20 {
            let mut f = 0.0005f64;
            for _ in 0..x {
                f /= 2f64;
            }
            let value = OctavePerlinNoiseSampler::maintain_precision(f);
            assert_eq!(value, *value_iter.next().unwrap());
        }
    }

    #[test]
    fn test_calculate_amplitudes() {
        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());

        assert_eq!(first, -15);
        assert_eq!(
            amplitudes,
            [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
        );
    }

    #[test]
    fn test_map() {
        let expected_data: Vec<i32> = read_data_from_file!("../../../assets/perlin_map.json");
        let mut expected_iter = expected_data.iter();

        let mut rand = Xoroshiro::from_seed(0);
        let splitter = rand.next_splitter();
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));
        assert_eq!(rand.next_i32(), 1374487555);
        let mut rand = RandomGenerator::Xoroshiro(splitter.split_string("minecraft:terrain"));

        let (first, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
        let sampler = OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true);
        let sampler = sampler.get_octave(0).unwrap();

        for x in -512..512 {
            let y = sampler.map(x);
            assert_eq!(y, *expected_iter.next().unwrap());
        }
    }
}
