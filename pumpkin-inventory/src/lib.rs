use crate::container_click::MouseClick;
use crate::player::PlayerInventory;
use num_derive::FromPrimitive;
use pumpkin_macros::screen;
use pumpkin_world::item::ItemStack;

pub mod container_click;
mod crafting;
pub mod drag_handler;
mod error;
mod open_container;
pub mod player;
pub mod window_property;

pub use error::InventoryError;
pub use open_container::*;

/// https://wiki.vg/Inventory
#[derive(Debug, FromPrimitive, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum WindowType {
    // not used
    Generic9x1 = screen!("minecraft:generic_9x1"),
    // not used
    Generic9x2 = screen!("minecraft:generic_9x2"),
    // General-purpose 3-row inventory. Used by Chest, minecart with chest, ender chest, and barrel
    Generic9x3 = screen!("minecraft:generic_9x3"),
    // not used
    Generic9x4 = screen!("minecraft:generic_9x4"),
    // not used
    Generic9x5 = screen!("minecraft:generic_9x5"),
    // Used by large chests
    Generic9x6 = screen!("minecraft:generic_9x6"),
    // General-purpose 3-by-3 square inventory, used by Dispenser and Dropper
    Generic3x3 = screen!("minecraft:generic_3x3"),
    // General-purpose 3-by-3 square inventory, used by the Crafter
    Craft3x3 = screen!("minecraft:crafter_3x3"),
    Anvil = screen!("minecraft:anvil"),
    Beacon = screen!("minecraft:beacon"),
    BlastFurnace = screen!("minecraft:blast_furnace"),
    BrewingStand = screen!("minecraft:brewing_stand"),
    CraftingTable = screen!("minecraft:crafting"),
    EnchantmentTable = screen!("minecraft:enchantment"),
    Furnace = screen!("minecraft:furnace"),
    Grindstone = screen!("minecraft:grindstone"),
    // Hopper or minecart with hopper
    Hopper = screen!("minecraft:hopper"),
    Lectern = screen!("minecraft:lectern"),
    Loom = screen!("minecraft:loom"),
    // Villager, Wandering Trader
    Merchant = screen!("minecraft:merchant"),
    ShulkerBox = screen!("minecraft:shulker_box"),
    SmithingTable = screen!("minecraft:smithing"),
    Smoker = screen!("minecraft:smoker"),
    CartographyTable = screen!("minecraft:cartography_table"),
    Stonecutter = screen!("minecraft:stonecutter"),
}
pub struct ContainerStruct<const SLOTS: usize>([Option<ItemStack>; SLOTS]);

// Container needs Sync + Send to be able to be in async Server
pub trait Container: Sync + Send {
    fn window_type(&self) -> &'static WindowType;

    fn window_name(&self) -> &'static str;

    fn handle_item_change(
        &mut self,
        carried_item: &mut Option<ItemStack>,
        slot: usize,
        mouse_click: MouseClick,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut all_slots = self.all_slots();
        if slot > all_slots.len() {
            Err(InventoryError::InvalidSlot)?
        }
        if taking_crafted {
            match (all_slots[slot].as_mut(), carried_item.as_mut()) {
                (Some(s1), Some(s2)) => {
                    if s1.item_id == s2.item_id {
                        handle_item_change(all_slots[slot], carried_item, mouse_click);
                    }
                }
                (Some(_), None) => handle_item_change(all_slots[slot], carried_item, mouse_click),
                (None, None) | (None, Some(_)) => (),
            }
            return Ok(());
        }
        handle_item_change(carried_item, all_slots[slot], mouse_click);

        Ok(())
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>>;

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>>;

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.all_slots_ref()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.all_slots()
    }

    fn internal_pumpkin_id(&self) -> u64 {
        0
    }

    fn craft(&mut self) -> bool {
        false
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        None
    }

    fn slot_in_crafting_input_slots(&self, _slot: &usize) -> bool {
        false
    }

    fn crafted_item_slot(&self) -> Option<ItemStack> {
        self.all_slots_ref()
            .get(self.crafting_output_slot()?)?
            .copied()
    }

    fn recipe_used(&mut self) {}
}

pub fn handle_item_take(
    carried_item: &mut Option<ItemStack>,
    item_slot: &mut Option<ItemStack>,
    mouse_click: MouseClick,
) {
    let Some(item) = item_slot else {
        return;
    };
    let mut new_item = *item;

    match mouse_click {
        MouseClick::Left => {
            *item_slot = None;
        }
        MouseClick::Right => {
            let half = item.item_count / 2;
            item.item_count -= half;
            new_item.item_count = half;
        }
    }
    *carried_item = Some(new_item);
}
pub fn handle_item_change(
    carried_slot: &mut Option<ItemStack>,
    current_slot: &mut Option<ItemStack>,
    mouse_click: MouseClick,
) {
    match (current_slot.as_mut(), carried_slot.as_mut()) {
        // Swap or combine current and carried
        (Some(current), Some(carried)) => {
            if current.item_id == carried.item_id {
                combine_stacks(carried_slot, current, mouse_click);
            } else if mouse_click == MouseClick::Left {
                let carried = *carried;
                *carried_slot = Some(current.to_owned());
                *current_slot = Some(carried.to_owned());
            }
        }
        // Put held stack into empty slot
        (None, Some(carried)) => match mouse_click {
            MouseClick::Left => {
                *current_slot = Some(carried.to_owned());
                *carried_slot = None;
            }
            MouseClick::Right => {
                carried.item_count -= 1;
                let mut new = *carried;
                new.item_count = 1;
                *current_slot = Some(new);
            }
        },
        // Take stack into carried
        (Some(_current), None) => handle_item_take(carried_slot, current_slot, mouse_click),
        (None, None) => (),
    }
}

pub fn combine_stacks(
    carried_slot: &mut Option<ItemStack>,
    slot: &mut ItemStack,
    mouse_click: MouseClick,
) {
    let Some(carried_item) = carried_slot else {
        return;
    };

    let carried_change = match mouse_click {
        MouseClick::Left => carried_item.item_count,
        MouseClick::Right => 1,
    };

    // TODO: Check for item stack max size here
    if slot.item_count + carried_change <= 64 {
        slot.item_count += carried_change;
        carried_item.item_count -= carried_change;
        if carried_item.item_count == 0 {
            *carried_slot = None;
        }
    } else {
        let left_over = slot.item_count + carried_change - 64;
        slot.item_count = 64;
        carried_item.item_count = left_over;
    }
}

pub struct OptionallyCombinedContainer<'a, 'b> {
    container: Option<&'a mut Box<dyn Container>>,
    inventory: &'b mut PlayerInventory,
}
impl<'a, 'b> OptionallyCombinedContainer<'a, 'b> {
    pub fn new(
        player_inventory: &'b mut PlayerInventory,
        container: Option<&'a mut Box<dyn Container>>,
    ) -> Self {
        Self {
            inventory: player_inventory,
            container,
        }
    }
    /// Returns None if the slot is in the players inventory, Returns Some(Option<&ItemStack>) if it's inside of the container
    pub fn get_slot_excluding_inventory(&self, slot: usize) -> Option<Option<&ItemStack>> {
        self.container.as_ref()?.all_slots_ref().get(slot).copied()
    }
}

impl<'a> Container for OptionallyCombinedContainer<'a, 'a> {
    fn window_type(&self) -> &'static WindowType {
        if let Some(container) = &self.container {
            container.window_type()
        } else {
            &WindowType::Generic9x1
        }
    }

    fn window_name(&self) -> &'static str {
        self.container
            .as_ref()
            .map(|container| container.window_name())
            .unwrap_or(self.inventory.window_name())
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let slots = match &mut self.container {
            Some(container) => {
                let mut slots = container.all_slots();
                slots.extend(self.inventory.all_combinable_slots_mut());
                slots
            }
            None => self.inventory.all_slots(),
        };
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        match &self.container {
            Some(container) => {
                let mut slots = container.all_slots_ref();
                slots.extend(self.inventory.all_combinable_slots());
                slots
            }
            None => self.inventory.all_slots_ref(),
        }
    }

    fn craft(&mut self) -> bool {
        match &mut self.container {
            Some(container) => container.craft(),
            None => self.inventory.craft(),
        }
    }

    fn crafting_output_slot(&self) -> Option<usize> {
        match &self.container {
            Some(container) => container.crafting_output_slot(),
            None => self.inventory.crafting_output_slot(),
        }
    }

    fn slot_in_crafting_input_slots(&self, slot: &usize) -> bool {
        match &self.container {
            Some(container) => {
                // We don't have to worry about length due to inventory crafting slots being inaccessible
                // while inside container interfaces
                container.slot_in_crafting_input_slots(slot)
            }
            None => self.inventory.slot_in_crafting_input_slots(slot),
        }
    }

    fn recipe_used(&mut self) {
        match &mut self.container {
            Some(container) => container.recipe_used(),
            None => self.inventory.recipe_used(),
        }
    }
}
