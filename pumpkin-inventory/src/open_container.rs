use crate::{Container, WindowType};
use pumpkin_world::item::ItemStack;
use std::sync::Mutex;

pub struct OpenContainer {
    players: Vec<i32>,
    container: Mutex<Box<dyn Container>>,
}

impl OpenContainer {
    pub fn try_open(&self, player_id: i32) -> Option<&Mutex<Box<dyn Container>>> {
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
            container: Mutex::new(Box::new(Chest::new())),
        }
    }

    pub fn all_player_ids(&self) -> Vec<i32> {
        self.players.clone()
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
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x3
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        dbg!(self.slots.iter().len());
        self.slots.iter_mut().collect()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.slots.iter().map(|slot| slot.as_ref()).collect()
    }

    fn advance_state_id(&mut self) -> i32 {
        self.state_id += 1;
        self.state_id - 1
    }

    fn reset_state_id(&mut self) {
        self.state_id = 0;
    }

    fn state_id(&self) -> i32 {
        self.state_id
    }
}
