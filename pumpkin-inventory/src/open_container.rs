use crate::crafting::check_if_matches_crafting;
use crate::player::PlayerInventory;
use crate::{Container, WindowType};
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::ItemStack;
use rand::random;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct ContainerHolder {
    pub containers_by_id: HashMap<usize, OpenContainer>,
    pub location_to_container_id: HashMap<WorldPosition, usize>,
}

impl ContainerHolder {
    pub async fn destroy(
        &mut self,
        id: usize,
        player_inventory: &mut PlayerInventory,
        carried_item: &mut Option<ItemStack>,
    ) -> Vec<Uuid> {
        if let Some(container) = self.containers_by_id.remove(&id) {
            let unique = container.unique;
            let players = container.players;
            let mut container = container.container.lock().await;
            container.destroy_container(player_inventory, carried_item, unique);
            players
        } else {
            vec![]
        }
    }

    pub async fn destroy_by_location(
        &mut self,
        location: &WorldPosition,
        player_inventory: &mut PlayerInventory,
        carried_item: &mut Option<ItemStack>,
    ) -> Vec<Uuid> {
        if let Some(id) = self.location_to_container_id.remove(location) {
            self.destroy(id, player_inventory, carried_item).await
        } else {
            vec![]
        }
    }

    pub fn get_by_location(&self, location: &WorldPosition) -> Option<&OpenContainer> {
        self.containers_by_id
            .get(self.location_to_container_id.get(location)?)
    }

    pub fn get_mut_by_location(&mut self, location: &WorldPosition) -> Option<&mut OpenContainer> {
        self.containers_by_id
            .get_mut(self.location_to_container_id.get(location)?)
    }

    pub fn new_by_location<C: Container + Default + 'static>(
        &mut self,
        location: WorldPosition,
        block: Option<Block>,
    ) -> Option<&mut OpenContainer> {
        if self.location_to_container_id.contains_key(&location) {
            return None;
        }
        let id = self.new_container::<C>(block, false);
        self.location_to_container_id.insert(location, id);
        self.containers_by_id.get_mut(&id)
    }

    pub fn new_container<C: Container + Default + 'static>(
        &mut self,
        block: Option<Block>,
        unique: bool,
    ) -> usize {
        let mut id: usize = random();
        let mut new_container = OpenContainer::new_empty_container::<C>(block, unique);
        while let Some(container) = self.containers_by_id.insert(id, new_container) {
            new_container = container;
            id = random();
        }
        id
    }

    pub fn new_unique<C: Container + Default + 'static>(
        &mut self,
        block: Option<Block>,
        player_id: Uuid,
    ) -> usize {
        let id = self.new_container::<C>(block, true);
        let container = self.containers_by_id.get_mut(&id).expect("just created it");
        container.players.push(player_id);
        id
    }
}

pub struct OpenContainer {
    pub unique: bool,
    block: Option<Block>,
    pub id: usize,
    container: Arc<Mutex<Box<dyn Container>>>,
    players: Vec<Uuid>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: Uuid) -> Option<&Arc<Mutex<Box<dyn Container>>>> {
        if !self.players.contains(&player_id) {
            log::debug!("couldn't open container");
            return None;
        }
        let container = &self.container;
        Some(container)
    }

    pub fn add_player(&mut self, player_id: Uuid) {
        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }
    }

    pub fn remove_player(&mut self, player_id: Uuid) {
        if let Some(index) = self.players.iter().enumerate().find_map(|(index, id)| {
            if *id == player_id {
                Some(index)
            } else {
                None
            }
        }) {
            self.players.remove(index);
        }
    }

    pub fn new_empty_container<C: Container + Default + 'static>(
        block: Option<Block>,
        unique: bool,
    ) -> Self {
        Self {
            unique,
            players: vec![],
            container: Arc::new(Mutex::new(Box::new(C::default()))),
            block,
            id: 0,
        }
    }

    pub fn clear_all_players(&mut self) {
        self.players = vec![];
    }

    pub fn all_player_ids(&self) -> &[Uuid] {
        &self.players
    }

    pub fn get_block(&self) -> Option<Block> {
        self.block.clone()
    }

    pub async fn window_type(&self) -> &'static WindowType {
        let container = self.container.lock().await;
        container.window_type()
    }
}
#[derive(Default)]
pub struct Chest([Option<ItemStack>; 27]);

impl Chest {
    pub fn new() -> Self {
        Self([None; 27])
    }
}
impl Container for Chest {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x3
    }

    fn window_name(&self) -> &'static str {
        "Chest"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.0.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.0.iter().map(|slot| slot.as_ref()).collect()
    }
}

#[derive(Default)]
pub struct CraftingTable {
    input: [[Option<ItemStack>; 3]; 3],
    output: Option<ItemStack>,
}

impl Container for CraftingTable {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::CraftingTable
    }

    fn window_name(&self) -> &'static str {
        "Crafting Table"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let slots = vec![&mut self.output];
        let slots = slots
            .into_iter()
            .chain(self.input.iter_mut().flatten())
            .collect();
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        let slots = vec![self.output.as_ref()];
        let slots = slots
            .into_iter()
            .chain(self.input.iter().flatten().map(|i| i.as_ref()))
            .collect();
        slots
    }

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.input.iter().flatten().map(|s| s.as_ref()).collect()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.input.iter_mut().flatten().collect()
    }

    fn craft(&mut self) -> bool {
        let old_output = self.output;
        self.output = check_if_matches_crafting(self.input);
        old_output != self.output
            || self.input.iter().flatten().any(|s| s.is_some())
            || self.output.is_some()
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(0)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        (1..10).contains(slot)
    }
    fn recipe_used(&mut self) {
        self.input.iter_mut().flatten().for_each(|slot| {
            if let Some(item) = slot {
                if item.item_count > 1 {
                    item.item_count -= 1;
                } else {
                    *slot = None;
                }
            }
        })
    }
}

#[derive(Default)]
pub struct Furnace {
    cook: Option<ItemStack>,
    fuel: Option<ItemStack>,
    output: Option<ItemStack>,
}

impl Container for Furnace {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Furnace
    }

    fn window_name(&self) -> &'static str {
        "Furnace"
    }
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let mut slots = vec![&mut self.cook];
        slots.push(&mut self.fuel);
        slots.push(&mut self.output);
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        let mut slots = vec![self.cook.as_ref()];
        slots.push(self.fuel.as_ref());
        slots.push(self.output.as_ref());
        slots
    }
}
