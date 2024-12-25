use crate::container_click::MouseClick;
use crate::crafting::check_if_matches_crafting;
use crate::{handle_item_change, Container, InventoryError, WindowType};
use pumpkin_world::item::ItemStack;
use std::iter::Chain;
use std::slice::IterMut;

pub struct PlayerInventory {
    // Main Inventory + Hotbar
    crafting: [Option<ItemStack>; 4],
    crafting_output: Option<ItemStack>,
    items: [Option<ItemStack>; 36],
    armor: [Option<ItemStack>; 4],
    offhand: Option<ItemStack>,
    // current selected slot in hotbar
    selected: usize,
    pub state_id: u32,
    // Notchian server wraps this value at 100, we can just keep it as a u8 that automatically wraps
    pub total_opened_containers: i32,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInventory {
    pub const CONTAINER_ID: i8 = 0;

    pub fn new() -> Self {
        Self {
            crafting: [None; 4],
            crafting_output: None,
            items: [None; 36],
            armor: [None; 4],
            offhand: None,
            // TODO: What when player spawns in with an different index ?
            selected: 0,
            state_id: 0,
            total_opened_containers: 2,
        }
    }
    /// Set the contents of an item in a slot
    ///
    /// ## Item
    /// The optional item to place in the slot
    ///
    /// ## Item allowed override
    /// An override, which when enabled, makes it so that invalid items, can be placed in slots they normally can't.
    /// Useful functionality for plugins in the future.
    pub fn set_slot(
        &mut self,
        slot: usize,
        item: Option<ItemStack>,
        item_allowed_override: bool,
    ) -> Result<(), InventoryError> {
        if item_allowed_override {
            if !(0..=45).contains(&slot) {
                Err(InventoryError::InvalidSlot)?
            }
            *self.all_slots()[slot] = item;
            return Ok(());
        }
        let slot_condition = self.slot_condition(slot)?;
        if let Some(item) = item {
            if slot_condition(&item) {
                *self.all_slots()[slot] = Some(item);
            }
        }
        Ok(())
    }
    #[allow(clippy::type_complexity)]
    pub fn slot_condition(
        &self,
        slot: usize,
    ) -> Result<Box<dyn Fn(&ItemStack) -> bool>, InventoryError> {
        if !(0..=45).contains(&slot) {
            return Err(InventoryError::InvalidSlot);
        }

        Ok(Box::new(match slot {
            0..=4 | 9..=45 => |_| true,
            5 => |item: &ItemStack| item.is_helmet(),
            6 => |item: &ItemStack| item.is_chestplate(),
            7 => |item: &ItemStack| item.is_leggings(),
            8 => |item: &ItemStack| item.is_boots(),
            _ => unreachable!(),
        }))
    }
    pub fn get_slot(&mut self, slot: usize) -> Result<&mut Option<ItemStack>, InventoryError> {
        match slot {
            0 => {
                // TODO: Add crafting check here
                Ok(&mut self.crafting_output)
            }
            1..=4 => Ok(&mut self.crafting[slot - 1]),
            5..=8 => Ok(&mut self.armor[slot - 5]),
            9..=44 => Ok(&mut self.items[slot - 9]),
            45 => Ok(&mut self.offhand),
            _ => Err(InventoryError::InvalidSlot),
        }
    }
    pub fn set_selected(&mut self, slot: usize) {
        assert!((0..9).contains(&slot));
        self.selected = slot;
    }

    pub fn get_selected(&self) -> usize {
        self.selected + 36
    }

    pub fn held_item(&self) -> Option<&ItemStack> {
        debug_assert!((0..9).contains(&self.selected));
        self.items[self.selected + 36 - 9].as_ref()
    }

    pub fn held_item_mut(&mut self) -> &mut Option<ItemStack> {
        debug_assert!((0..9).contains(&self.selected));
        &mut self.items[self.selected + 36 - 9]
    }

    pub fn slots(&self) -> Vec<Option<&ItemStack>> {
        let mut slots = vec![self.crafting_output.as_ref()];
        slots.extend(self.crafting.iter().map(|c| c.as_ref()));
        slots.extend(self.armor.iter().map(|c| c.as_ref()));
        slots.extend(self.items.iter().map(|c| c.as_ref()));
        slots.push(self.offhand.as_ref());
        slots
    }

    pub fn slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        let mut slots = vec![&mut self.crafting_output];
        slots.extend(self.crafting.iter_mut());
        slots.extend(self.armor.iter_mut());
        slots.extend(self.items.iter_mut());
        slots.push(&mut self.offhand);
        slots
    }

    pub fn iter_items_mut(&mut self) -> IterMut<Option<ItemStack>> {
        self.items.iter_mut()
    }

    pub fn slots_with_hotbar_first(
        &mut self,
    ) -> Chain<IterMut<Option<ItemStack>>, IterMut<Option<ItemStack>>> {
        let (items, hotbar) = self.items.split_at_mut(27);
        hotbar.iter_mut().chain(items)
    }
}

impl Container for PlayerInventory {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x1
    }

    fn window_name(&self) -> &'static str {
        // We never send an OpenContainer with inventory, so it has no name.
        ""
    }

    fn handle_item_change(
        &mut self,
        carried_slot: &mut Option<ItemStack>,
        slot: usize,
        mouse_click: MouseClick,
        invert: bool,
    ) -> Result<(), InventoryError> {
        let slot_condition = self.slot_condition(slot)?;
        let item_slot = self.get_slot(slot)?;
        if let Some(item) = carried_slot {
            if slot_condition(item) {
                if invert {
                    handle_item_change(item_slot, carried_slot, mouse_click);
                    return Ok(());
                }
                handle_item_change(carried_slot, item_slot, mouse_click);
            }
        } else {
            if invert {
                handle_item_change(item_slot, carried_slot, mouse_click);
                return Ok(());
            }
            handle_item_change(carried_slot, item_slot, mouse_click)
        }
        Ok(())
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.slots_mut()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.slots()
    }

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|item| item.as_ref()).collect()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }

    fn craft(&mut self) -> bool {
        let v1 = [self.crafting[0], self.crafting[1], None];
        let v2 = [self.crafting[2], self.crafting[3], None];
        let v3 = [None; 3];
        let together = [v1, v2, v3];

        self.crafting_output = check_if_matches_crafting(together);
        self.crafting.iter().any(|s| s.is_some())
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        Some(0)
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        (1..=4).contains(slot)
    }
}
