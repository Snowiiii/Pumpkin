use crate::container_click::MouseClick;
use crate::{handle_item_change, Container, WindowType};
use pumpkin_world::item::ItemStack;
use std::sync::{Mutex, MutexGuard};

pub struct OpenContainer {
    players: Vec<i32>,
    container: Mutex<Box<dyn Container>>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: i32) -> Option<MutexGuard<Box<dyn Container>>> {
        if !self.players.contains(&player_id) {
            return None;
        }
        Some(self.container.lock().unwrap())
    }

    pub fn empty(player_id: i32) -> Self {
        Self {
            players: vec![player_id],
            container: Mutex::new(Box::new(Chest::new())),
        }
    }
}

struct Chest([Option<ItemStack>; 27]);

impl Chest {
    pub fn new() -> Self {
        Self([None; 27])
    }
}
impl Container for Chest {
    fn window_type(&self) -> &WindowType {
        &WindowType::Generic9x3
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.0.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.0.iter().map(|slot| slot.as_ref()).collect()
    }
}
