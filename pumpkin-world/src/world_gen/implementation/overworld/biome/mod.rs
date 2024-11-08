use std::ops::Add;

use crate::{block::BlockState, coordinates::ChunkRelativeBlockCoordinates};

pub mod plains;
pub mod void;

pub fn generate_tree(
    chunk_relative_coordinates: ChunkRelativeBlockCoordinates,
) -> Vec<(ChunkRelativeBlockCoordinates, BlockState)> {
    let x = chunk_relative_coordinates.x;
    let z = chunk_relative_coordinates.z;

    // TODO: Adjust tree height and trunk width based on biome
    let tree_height: i8 = 7;
    let trunk_width: i8 = 1;

    let mut tree_blocks = Vec::new();

    // Generate trunk
    for y in 0..tree_height {
        for dx in 0 - trunk_width..=trunk_width {
            for dz in 0 - trunk_width..=trunk_width {
                let block_coordinates = ChunkRelativeBlockCoordinates {
                    x: x.add(dx as u8).into(),
                    y: (chunk_relative_coordinates.y.add(y as i16)).into(),
                    z: z.add(dz as u8).into(),
                };
                tree_blocks.push((
                    block_coordinates,
                    BlockState::new("minecraft:oak_log").unwrap(),
                ));
            }
        }
    }

    // Generate leaves
    let leaf_radius = trunk_width + 1;
    for y in tree_height..tree_height + 3 {
        for dx in 0 - leaf_radius..=leaf_radius {
            for dz in 0 - leaf_radius..=leaf_radius {
                let block_coordinates = ChunkRelativeBlockCoordinates {
                    x: x.add(dx as u8).into(),
                    y: (chunk_relative_coordinates.y.add(y as i16)).into(),
                    z: z.add(dz as u8).into(),
                };
                tree_blocks.push((
                    block_coordinates,
                    BlockState::new("minecraft:oak_leaves").unwrap(),
                ));
            }
        }
    }

    tree_blocks
}
