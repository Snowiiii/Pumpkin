use std::{collections::HashMap, sync::{Arc, OnceLock}};

use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::{block_registry::Property, BlockFace};
use pumpkin_world::block::block_registry::Block;

use crate::{block::block_properties_manager::{get_property_key, BlockBehavior, BlockProperty, SlabPosition}, world::World};

pub static SLAB_BEHAVIOR: OnceLock<Arc<SlabBehavior>> = OnceLock::new();

// Example of a behavior with shared static data
pub struct SlabBehavior {
  // Shared static data for all slabs
  state_mappings: HashMap<Vec<String>, u16>,
  property_mappings: HashMap<u16, Vec<String>>,
}

impl SlabBehavior {
    pub fn get_or_init(properties: &Vec<Property>) -> Arc<Self> {
        SLAB_BEHAVIOR.get_or_init(|| Arc::new(Self::new(properties))).clone()
    }

    pub fn get() -> Arc<Self> {
        SLAB_BEHAVIOR.get().expect("Slab Uninitialized").clone()
    }

    pub fn new(properties: &Vec<Property>) -> Self {
        let total_combinations: usize = properties
        .iter()
        .map(|p| p.values.len())
        .product();

        let mut forward_map = HashMap::with_capacity(total_combinations);
        let mut reverse_map = HashMap::with_capacity(total_combinations);
        
        for i in 0..total_combinations {
            let mut current = i;
            let mut combination = Vec::with_capacity(properties.len());
            
            for property in properties.iter().rev() {
                let property_size = property.values.len();
                combination.push(current % property_size);
                current /= property_size;
            }
            
            combination.reverse();
            
            let key: Vec<String> = combination
                .iter()
                .enumerate()
                .map(|(prop_idx, &state_idx)| {
                    format!("{}{}", 
                        properties[prop_idx].name,
                        properties[prop_idx].values[state_idx])
                })
                .collect();
            
            forward_map.insert(key.clone(), i as u16);
            reverse_map.insert(i as u16, key);
        }

        Self {
            state_mappings: forward_map,
            property_mappings: reverse_map,
        }
    }

    pub fn evalute_property_type(&self, block: &Block, clicked_block: &Block, world_pos: &WorldPosition, face: &BlockFace) -> String {
        if block.id == clicked_block.id && *face == BlockFace::Top {
            return format!("{}{}", "type", "double")
        }
        format!("{}{}", "type", "bottom")
    }

    pub fn evalute_property_waterlogged(&self, block: &Block, clicked_block: &Block, world_pos: &WorldPosition, face: &BlockFace) -> String {
        if clicked_block.name == "water" {
            return format!("{}{}", "waterlogged", "true")
        }
        return format!("{}{}", "waterlogged", "false")
    }
}

#[async_trait::async_trait]
impl BlockBehavior for SlabBehavior {
    async fn map_state_id(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> u16 {
        let clicked_block = world.get_block(*world_pos).await.unwrap();
        let mut hmap_key: Vec<String> = Vec::with_capacity(block.properties.len());
        let slab_behaviour = SlabBehavior::get();

        for property in block.properties.iter() {
            let state = match get_property_key(&property.name.as_str()).expect("Property not found") {
                BlockProperty::SlabType(SlabPosition::Top) => slab_behaviour.evalute_property_type(block, clicked_block, world_pos, face),
                BlockProperty::Waterlogged(false) => slab_behaviour.evalute_property_waterlogged(block, clicked_block, world_pos, face),
                _ => panic!("Property not found"),
            };
            hmap_key.push(state.to_string());
        }

        // Base state id plus offset
        block.states[0].id + slab_behaviour.state_mappings[&hmap_key]
    }

    async fn is_updateable(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> bool {
        let clicked_block = world.get_block(*world_pos).await.unwrap();
        if block.id != clicked_block.id || *face != BlockFace::Top {
            return false;
        }

        let clicked_block_state_id = world.get_block_state_id(*world_pos).await.unwrap();

        if let Some(properties) = SlabBehavior::get().property_mappings.get(&(&clicked_block_state_id - clicked_block.states[0].id)) {
            log::warn!("Properties: {:?}", properties);
            if properties.contains(&"typebottom".to_string()) {
                return true;
            }
        }
        false
    }
}