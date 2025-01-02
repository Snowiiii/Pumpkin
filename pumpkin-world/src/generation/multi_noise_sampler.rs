use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, Map};

use crate::biome::Biome;
use crate::generation::noise::density::component_functions::ComponentReference;

use super::chunk_noise::ChunkNoiseState;
use super::noise::density::component_functions::NoEnvironment;
use super::noise::density::NoisePos;

#[derive(Clone, Serialize, Deserialize)]
pub struct NoiseValuePoint {
    pub temperature: f64,
    pub erosion: f64,
    pub depth: f64,
    pub continents: f64,
    pub weirdness: f64,
    pub humidity: f64,
}

pub struct MultiNoiseSampler {
    pub(crate) temperature: Box<dyn ComponentReference<NoEnvironment>>,
    pub(crate) erosion: Box<dyn ComponentReference<NoEnvironment>>,
    pub(crate) depth: Box<dyn ComponentReference<NoEnvironment>>,
    pub(crate) continents: Box<dyn ComponentReference<NoEnvironment>>,
    pub(crate) weirdness: Box<dyn ComponentReference<NoEnvironment>>,
    pub(crate) humidity: Box<dyn ComponentReference<NoEnvironment>>,
}

impl MultiNoiseSampler {
    pub fn sample(&mut self, pos: &NoisePos) -> NoiseValuePoint {
        NoiseValuePoint {
            temperature: self.temperature.sample_mut(pos, &NoEnvironment {}),
            erosion: self.erosion.sample_mut(pos, &NoEnvironment {}),
            depth: self.depth.sample_mut(pos, &NoEnvironment {}),
            continents: self.continents.sample_mut(pos, &NoEnvironment {}),
            weirdness: self.weirdness.sample_mut(pos, &NoEnvironment {}),
            humidity: self.humidity.sample_mut(pos, &NoEnvironment {}),
        }
    }
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct BiomeEntries {
    #[serde_as(as = "Map<_, _>")]
    nodes: Vec<(Biome, NoiseValuePoint)>,
}

impl BiomeEntries {
    pub fn new(nodes: Vec<(Biome, NoiseValuePoint)>) -> Self {
        Self { nodes }
    }

    pub fn find_biome(&self, point: &NoiseValuePoint) -> Biome {
        let mut closest_biome = None;
        let mut min_distance = f64::MAX;

        for (biome, center) in &self.nodes {
            let distance = center.distance_squared(point);
            if distance < min_distance {
                min_distance = distance;
                closest_biome = Some(biome);
            }
        }

        closest_biome.cloned().unwrap_or(Biome::Plains) // Default biome if none matches.
    }
}

impl NoiseValuePoint {
    /// Calculates the squared distance between two NoiseValuePoints.
    pub fn distance_squared(&self, other: &Self) -> f64 {
        let temp_diff = self.temperature - other.temperature;
        let erosion_diff = self.erosion - other.erosion;
        let depth_diff = self.depth - other.depth;
        let continents_diff = self.continents - other.continents;
        let weirdness_diff = self.weirdness - other.weirdness;
        let humidity_diff = self.humidity - other.humidity;

        temp_diff * temp_diff
            + erosion_diff * erosion_diff
            + depth_diff * depth_diff
            + continents_diff * continents_diff
            + weirdness_diff * weirdness_diff
            + humidity_diff * humidity_diff
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_search_tree() {
        let biomes = vec![
            (
                Biome::Plains,
                NoiseValuePoint {
                    temperature: 0.8,
                    erosion: 0.3,
                    depth: 0.1,
                    continents: 0.5,
                    weirdness: 0.4,
                    humidity: 0.9,
                },
            ),
            (
                Biome::SnowyTiga,
                NoiseValuePoint {
                    temperature: -0.5,
                    erosion: 0.1,
                    depth: 0.2,
                    continents: 0.7,
                    weirdness: 0.2,
                    humidity: 0.4,
                },
            ),
        ];

        let search_tree = BiomeEntries::new(biomes);

        let query = NoiseValuePoint {
            temperature: 0.7,
            erosion: 0.3,
            depth: 0.1,
            continents: 0.5,
            weirdness: 0.4,
            humidity: 0.8,
        };

        let result = search_tree.find_biome(&query);
        assert_eq!(result, Biome::Plains);
    }
}
