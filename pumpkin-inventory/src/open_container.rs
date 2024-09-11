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
    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.0.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.0.iter().map(|slot| slot.as_ref()).collect()
    }
}
