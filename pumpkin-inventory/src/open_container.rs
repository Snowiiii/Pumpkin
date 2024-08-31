use crate::{Container, WindowType};
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
        let container = self.container.lock().unwrap();
        container
            .all_slots_ref()
            .iter()
            .enumerate()
            .for_each(|(slot, item)| {
                if let Some(item) = item {
                    dbg!(slot, item);
                }
            });
        Some(container)
    }

    pub fn add_player(&mut self, player_id: i32) {
        if !self.players.contains(&player_id) {
            self.players.push(player_id);
        }
    }

    pub fn empty(player_id: i32) -> Self {
        Self {
            players: vec![player_id],
            container: Mutex::new(Box::new(Chest::new())),
        }
    }
}

struct Chest {
    slots: [Option<ItemStack>; 27],
    state_id: i32,
}

impl Chest {
    pub fn new() -> Self {
        Self {
            slots: [None; 27],
            state_id: 0,
        }
    }
}
impl Container for Chest {
    fn window_type(&self) -> &WindowType {
        &WindowType::Generic9x3
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.slots.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.slots.iter().map(|slot| slot.as_ref()).collect()
    }

    fn state_id(&mut self) -> i32 {
        self.state_id += 1;
        self.state_id - 1
    }
}
