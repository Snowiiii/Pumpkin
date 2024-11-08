use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

// TODO make this work with the protocol
// TODO add missing biomes
// Send by the registry
#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
#[non_exhaustive]
pub enum Biome {
    Ocean = 0,
    Plains = 1,
    Desert = 2,
    ExtremeHills = 3,
    Forest = 4,
    Taiga = 5,
    Swampland = 6,
    River = 7,
    Hell = 8,
    Sky = 9,
    FrozenOcean = 10,
    FrozenRiver = 11,
    IceFlats = 12,
    IceMountains = 13,
    MushroomIsland = 14,
    MushroomIslandShore = 15,
    Beaches = 16,
    DesertHills = 17,
    ForestHills = 18,
    TaigaHills = 19,
    SmallerExtremeHills = 20,
    Jungle = 21,
    JungleHills = 22,
    JungleEdge = 23,
    DeepOcean = 24,
    StoneBeach = 25,
    ColdBeach = 26,
    BirchForest = 27,
    BirchForestHills = 28,
    RoofedForest = 29,
    TaigaCold = 30,
    TaigaColdHills = 31,
    RedwoodTaiga = 32,
    RedwoodTaigaHills = 33,
    ExtremeHillsWithTrees = 34,
    Savanna = 35,
    SavannaRock = 36,
    Mesa = 37,
    MesaRock = 38,
    MesaClearRock = 39,
    #[default]
    Void = 127,
    MutatedPlains = 129,
    MutatedDesert = 130,
    MutatedExtremeHills = 131,
    MutatedForest = 132,
    MutatedTaiga = 133,
    MutatedSwampland = 134,
    MutatedIceFlats = 140,
    MutatedJungle = 149,
    MutatedJungleEdge = 151,
    MutatedBirchForest = 155,
    MutatedBirchForestHills = 156,
    MutatedRoofedForest = 157,
    MutatedTaigaCold = 158,
    MutatedRedwoodTaiga = 160,
    MutatedRedwoodTaigaHills = 161,
    MutatedExtremeHillsWithTrees = 162,
    MutatedSavanna = 163,
    MutatedSavannaRock = 164,
    MutatedMesa = 165,
    MutatedMesaRock = 166,
    MutatedMesaClearRock = 167,
}

#[derive(Clone)]
#[enum_dispatch(BiomeSupplierImpl)]
pub enum BiomeSupplier {
    Debug(DebugBiomeSupplier),
}

#[enum_dispatch]
pub trait BiomeSupplierImpl {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &MultiNoiseSampler) -> Biome;
}

#[derive(Clone)]
pub struct DebugBiomeSupplier {}

impl BiomeSupplierImpl for DebugBiomeSupplier {
    fn biome(&self, _x: i32, _y: i32, _z: i32, _noise: &MultiNoiseSampler) -> Biome {
        Biome::Plains
    }
}

// TODO: Implement
pub struct MultiNoiseSampler {}
