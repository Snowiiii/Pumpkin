use crate::container_click::MouseDragType;
use crate::{Container, InventoryError};
use num_traits::Euclid;
use pumpkin_world::item::ItemStack;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
#[derive(Debug, Default)]
pub struct DragHandler(RwLock<HashMap<u64, Arc<Mutex<Drag>>>>);

impl DragHandler {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub async fn new_drag(
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
        let mut drags = self.0.write().await;
        drags.insert(container_id, Arc::new(Mutex::new(drag)));
        Ok(())
    }

    pub async fn add_slot(
        &self,
        container_id: u64,
        player: i32,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let drags = self.0.read().await;
        match drags.get(&container_id) {
            Some(drag) => {
                let mut drag = drag.lock().await;
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

    pub async fn apply_drag<T: Container>(
        &self,
        maybe_carried_item: &mut Option<ItemStack>,
        container: &mut T,
        container_id: &u64,
        player: i32,
    ) -> Result<(), InventoryError> {
        // Minecraft client does still send dragging packets when not carrying an item!
        if maybe_carried_item.is_none() {
            return Ok(());
        }

        let mut drags = self.0.write().await;
        let Some((_, drag)) = drags.remove_entry(container_id) else {
            Err(InventoryError::OutOfOrderDragging)?
        };
        let drag = drag.lock().await;

        if player != drag.player {
            Err(InventoryError::MultiplePlayersDragging)?
        }
        let mut slots = container.all_slots();
        let slots_cloned: Vec<Option<ItemStack>> = slots
            .iter()
            .map(|stack| stack.map(|item| item.to_owned()))
            .collect();
        let Some(carried_item) = maybe_carried_item else {
            return Ok(());
        };
        match drag.drag_type {
            // This is only valid in Creative GameMode.
            // Checked in any function that uses this function.
            MouseDragType::Middle => {
                for slot in &drag.slots {
                    *slots[*slot] = *maybe_carried_item;
                }
            }
            MouseDragType::Right => {
                let mut single_item = *carried_item;
                single_item.item_count = 1;

                let changing_slots =
                    drag.possibly_changing_slots(&slots_cloned, carried_item.item_id);
                changing_slots.for_each(|slot| {
                    if carried_item.item_count != 0 {
                        carried_item.item_count -= 1;
                        if let Some(stack) = &mut slots[slot] {
                            // TODO: Check for stack max here
                            if stack.item_count + 1 < 64 {
                                stack.item_count += 1;
                            } else {
                                carried_item.item_count += 1;
                            }
                        } else {
                            *slots[slot] = Some(single_item)
                        }
                    }
                });

                if carried_item.item_count == 0 {
                    *maybe_carried_item = None
                }
            }
            MouseDragType::Left => {
                // TODO: Handle dragging a stack with greater amount than item allows as max unstackable
                // In that specific case, follow MouseDragType::Right behaviours instead!

                let changing_slots =
                    drag.possibly_changing_slots(&slots_cloned, carried_item.item_id);
                let amount_of_slots = changing_slots.clone().count();
                let (amount_per_slot, remainder) =
                    (carried_item.item_count as usize).div_rem_euclid(&amount_of_slots);
                let mut item_in_each_slot = *carried_item;
                item_in_each_slot.item_count = amount_per_slot as u8;
                changing_slots.for_each(|slot| *slots[slot] = Some(item_in_each_slot));

                if remainder > 0 {
                    carried_item.item_count = remainder as u8;
                } else {
                    *maybe_carried_item = None
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
    fn possibly_changing_slots<'a>(
        &'a self,
        slots: &'a [Option<ItemStack>],
        carried_item_id: u16,
    ) -> impl Iterator<Item = usize> + 'a + Clone {
        self.slots.iter().filter_map(move |slot_index| {
            let slot = &slots[*slot_index];

            match slot {
                Some(item_slot) => {
                    if item_slot.item_id == carried_item_id {
                        Some(*slot_index)
                    } else {
                        None
                    }
                }
                None => Some(*slot_index),
            }
        })
    }
}
