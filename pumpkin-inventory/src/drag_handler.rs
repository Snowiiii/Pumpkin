use crate::container_click::MouseDragType;
use crate::{Container, InventoryError};
use itertools::Itertools;
use num_traits::Euclid;
use pumpkin_world::item::ItemStack;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
#[derive(Debug, Default)]
pub struct DragHandler(RwLock<HashMap<u64, Arc<Mutex<Drag>>>>);

impl DragHandler {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn new_drag(
        &self,
        container_id: u64,
        player: i32,
        drag_type: MouseDragType,
    ) -> Result<(), InventoryError> {
        let drag = Drag {
            player,
            drag_type,
            slots: vec![],
        };
        let mut drags = match self.0.write() {
            Ok(drags) => drags,
            Err(_) => Err(InventoryError::LockError)?,
        };
        drags.insert(container_id, Arc::new(Mutex::new(drag)));
        Ok(())
    }

    pub fn add_slot(
        &self,
        container_id: u64,
        player: i32,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let drags = match self.0.read() {
            Ok(drags) => drags,
            Err(_) => Err(InventoryError::LockError)?,
        };
        match drags.get(&container_id) {
            Some(drag) => {
                let mut drag = drag.lock().unwrap();
                if drag.player != player {
                    Err(InventoryError::MultiplePlayersDragging)?
                }
                if !drag.slots.contains(&slot) {
                    drag.slots.push(slot);
                }
            }
            None => Err(InventoryError::OutOfOrderDragging)?,
        }
        Ok(())
    }

    pub fn apply_drag<T: Container>(
        &self,
        carried_item: &mut Option<ItemStack>,
        container: &mut T,
        container_id: &u64,
        player: i32,
    ) -> Result<(), InventoryError> {
        // Minecraft client does still send dragging packets when not carrying an item!
        if carried_item.is_none() {
            return Ok(());
        }

        let Ok(mut drags) = self.0.write() else {
            Err(InventoryError::LockError)?
        };
        let Some((_, drag)) = drags.remove_entry(container_id) else {
            Err(InventoryError::OutOfOrderDragging)?
        };
        let drag = drag.lock().unwrap();

        if player != drag.player {
            Err(InventoryError::MultiplePlayersDragging)?
        }
        let mut slots = container.all_slots();
        let slots_cloned = slots
            .iter()
            .map(|stack| stack.map(|item| item.to_owned()))
            .collect_vec();
        match drag.drag_type {
            // This is only valid in Creative GameMode.
            // Checked in any function that uses this function.
            MouseDragType::Middle => {
                for slot in &drag.slots {
                    *slots[*slot] = *carried_item;
                }
            }
            MouseDragType::Right => {
                let amount_of_items = carried_item.unwrap().item_count as usize;
                let mut single_item = carried_item.unwrap();
                single_item.item_count = 1;
                let changing_slots = drag.changing_slots(
                    amount_of_items,
                    &slots_cloned,
                    carried_item.as_ref().unwrap(),
                );
                let mut amount_removed = 0;
                changing_slots.for_each(|slot| {
                    amount_removed += 1;
                    *slots[slot] = Some(single_item)
                });

                let mut remaining = carried_item.unwrap();
                if remaining.item_count == amount_removed {
                    *carried_item = None
                } else {
                    remaining.item_count -= amount_removed;
                    *carried_item = Some(remaining)
                }
            }
            MouseDragType::Left => {
                let amount_of_items = carried_item.unwrap().item_count as usize;
                // TODO: Handle dragging a stack with greater amount than item allows as max unstackable
                // In that specific case, follow MouseDragType::Right behaviours instead!

                let changing_slots = drag.changing_slots(
                    amount_of_items,
                    &slots_cloned,
                    carried_item.as_ref().unwrap(),
                );
                let amount_of_slots = changing_slots.clone().count();
                let (amount_per_slot, remainder) =
                    (carried_item.unwrap().item_count as usize).div_rem_euclid(&amount_of_slots);
                let mut item_in_each_slot = carried_item.unwrap();
                item_in_each_slot.item_count = amount_per_slot as u8;
                changing_slots.for_each(|slot| *slots[slot] = Some(item_in_each_slot));

                if remainder > 0 {
                    let mut remaining = carried_item.unwrap();
                    remaining.item_count = remainder as u8;
                    *carried_item = Some(remaining)
                } else {
                    *carried_item = None
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug)]
struct Drag {
    player: i32,
    drag_type: MouseDragType,
    slots: Vec<usize>,
}

impl Drag {
    fn changing_slots<'a>(
        &'a self,
        amount_of_items: usize,
        slots: &'a [Option<ItemStack>],
        carried_item: &'a ItemStack,
    ) -> impl Iterator<Item = usize> + 'a + Clone {
        self.slots
            .iter()
            .enumerate()
            .take_while(move |(slot_number, _)| *slot_number <= amount_of_items)
            .filter_map(move |(_, slot)| match &slots[*slot] {
                Some(item_slot) => {
                    if *item_slot == *carried_item {
                        Some(*slot)
                    } else {
                        None
                    }
                }
                None => Some(*slot),
            })
    }
}
