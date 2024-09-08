use crate::{Container, WindowType};
use parking_lot::Mutex;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

pub struct OpenContainer {
    players: Vec<i32>,
    container: Arc<Mutex<Box<dyn Container>>>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: i32) -> Option<&Arc<Mutex<Box<dyn Container>>>> {
        if !self.players.contains(&player_id) {
            dbg!("couldn't open container");
            return None;
        }
        let container = &self.container;
        Some(container)
    }

    pub fn add_player(&mut self, player_id: i32) {
        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }
    }

    pub fn remove_player(&mut self, player_id: i32) {
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

    pub fn empty(player_id: i32) -> Self {
        Self {
            players: vec![player_id],
            container: Arc::new(Mutex::new(Box::new(Chest::new()))),
        }
    }

    pub fn all_player_ids(&self) -> Vec<i32> {
        self.players.clone()
    }
}

struct Chest([Option<ItemStack>; 27]);

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
    fn iter_slots_mut<'s>(
        &'s mut self,
    ) -> Box<(dyn ExactSizeIterator<Item = &'s mut Option<ItemStack>> + 's)> {
        Box::new(self.0.iter_mut())
    }

    fn iter_slots<'s>(
        &'s self,
    ) -> Box<(dyn ExactSizeIterator<Item = &'s Option<ItemStack>> + 's)> {
        Box::new(self.0.iter())
    }

    fn get_slot(&self, slot: usize) -> Option<&Option<ItemStack>> {
        todo!()
    }

    fn get_slot_mut(&mut self, slot: usize) -> Option<&mut Option<ItemStack>> {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }
}
