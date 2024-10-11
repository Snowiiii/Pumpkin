use crate::container_click::MouseClick;
use crate::player::PlayerInventory;
use itertools::Itertools;
use num_derive::{FromPrimitive, ToPrimitive};
use pumpkin_world::item::ItemStack;

pub mod container_click;
pub mod drag_handler;
mod error;
mod open_container;
pub mod player;
pub mod window_property;

pub use error::InventoryError;
pub use open_container::OpenContainer;

/// https://wiki.vg/Inventory
#[derive(Debug, ToPrimitive, FromPrimitive, Clone, Copy, Eq, PartialEq)]
pub enum WindowType {
    // not used
    Generic9x1,
    // not used
    Generic9x2,
    // General-purpose 3-row inventory. Used by Chest, minecart with chest, ender chest, and barrel
    Generic9x3,
    // not used
    Generic9x4,
    // not used
    Generic9x5,
    // Used by large chests
    Generic9x6,
    // General-purpose 3-by-3 square inventory, used by Dispenser and Dropper
    Generic3x3,
    // General-purpose 3-by-3 square inventory, used by the Crafter
    Craft3x3,
    Anvil,
    Beacon,
    BlastFurnace,
    BrewingStand,
    CraftingTable,
    EnchantmentTable,
    Furnace,
    Grindstone,
    // Hopper or minecart with hopper
    Hopper,
    Lectern,
    Loom,
    // Villager, Wandering Trader
    Merchant,
    ShulkerBox,
    SmithingTable,
    Smoker,
    CartographyTable,
    Stonecutter,
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
    ) -> Result<(), InventoryError> {
        handle_item_change(
            carried_item,
            self.get_slot_mut(slot).ok_or(InventoryError::InvalidSlot)?,
            mouse_click,
        );
        Ok(())
    }

    fn iter_slots_mut<'s>(&'s mut self) -> Box<dyn Iterator<Item = &mut Option<ItemStack>> + 's>;

    fn iter_slots<'s>(&'s self) -> Box<dyn Iterator<Item = &Option<ItemStack>> + 's>;

    fn iter_combinable_slots<'s>(&'s self) -> Box<dyn Iterator<Item = &Option<ItemStack>> + 's> {
        self.iter_slots()
    }

    fn iter_combinable_slots_mut<'s>(
        &'s mut self,
    ) -> Box<dyn Iterator<Item = &mut Option<ItemStack>> + 's> {
        self.iter_slots_mut()
    }

    fn internal_pumpkin_id(&self) -> u64 {
        0
    }

    /// Returns a reference to the item stack in the specified slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The index of the slot to check
    ///
    /// # Returns
    ///
    /// * `Some(Some(ItemStack))` - If the slot exists and contains an item
    /// * `Some(None)` - If the slot exists but is empty
    /// * `None` - If the slot does not exist
    fn get_slot(&self, slot: usize) -> Option<&Option<ItemStack>>;

    /// Returns a mutable reference to the item stack in the specified slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The index of the slot to check
    ///
    /// # Returns
    ///
    /// * `Some(Some(ItemStack))` - If the slot exists and contains an item
    /// * `Some(None)` - If the slot exists but is empty
    /// * `None` - If the slot does not exist
    fn get_slot_mut(&mut self, slot: usize) -> Option<&mut Option<ItemStack>>;

    fn size(&self) -> usize;
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
    pub fn get_slot_excluding_inventory(&self, slot: usize) -> Option<&Option<ItemStack>> {
        if let Some(container) = &self.container {
            return container.get_slot(slot);
        }
        None
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
        let Some(container) = &self.container else {
            return self.inventory.window_name();
        };
        container.window_name()
    }

    fn iter_slots_mut<'s>(&'s mut self) -> Box<(dyn Iterator<Item = &mut Option<ItemStack>> + 's)> {
        match &mut self.container {
            Some(container) => Box::new(
                container
                    .iter_slots_mut()
                    .chain(self.inventory.iter_combinable_slots_mut()),
            ),
            None => self.inventory.iter_slots_mut(),
        }
    }

    fn iter_slots<'s>(&'s self) -> Box<(dyn Iterator<Item = &Option<ItemStack>> + 's)> {
        match &self.container {
            Some(container) => Box::new(
                container
                    .iter_slots()
                    .chain(self.inventory.iter_combinable_slots()),
            ),
            None => self.inventory.iter_slots(),
        }
    }

    fn get_slot(&self, slot: usize) -> Option<&Option<ItemStack>> {
        match &self.container {
            Some(container) => {
                if (0..container.size()).contains(&slot) {
                    container.get_slot(slot)
                } else if (container.size()..(container.size() + 27)).contains(&slot) {
                    self.inventory.get_slot(slot - container.size())
                } else {
                    None
                }
            }
            None => self.inventory.get_slot(slot),
        }
    }

    fn get_slot_mut(&mut self, slot: usize) -> Option<&mut Option<ItemStack>> {
        match &mut self.container {
            Some(container) => {
                if (0..container.size()).contains(&slot) {
                    container.get_slot_mut(slot)
                } else if (container.size()..(container.size() + 27)).contains(&slot) {
                    self.inventory.get_slot_mut(slot - container.size())
                } else {
                    None
                }
            }
            None => self.inventory.get_slot_mut(slot),
        }
    }

    fn size(&self) -> usize {
        self.inventory.size() + self.container.as_ref().map_or(0, |c| c.size())
    }
}
