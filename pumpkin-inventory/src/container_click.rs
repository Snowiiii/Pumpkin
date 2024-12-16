use crate::InventoryError;
use pumpkin_protocol::server::play::SlotActionType;
use pumpkin_world::item::ItemStack;

pub struct Click {
    pub slot: Slot,
    pub click_type: ClickType,
}

impl Click {
    pub fn new(mode: SlotActionType, button: i8, slot: i16) -> Result<Self, InventoryError> {
        match mode {
            SlotActionType::Pickup => Self::new_normal_click(button, slot),
            // Both buttons do the same here, so we omit it
            SlotActionType::QuickMove => Self::new_shift_click(slot),
            SlotActionType::Swap => Self::new_key_click(button, slot),
            SlotActionType::Clone => Ok(Self {
                click_type: ClickType::CreativePickItem,
                slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            }),
            SlotActionType::Throw => Self::new_drop_item(button),
            SlotActionType::QuickCraft => Self::new_drag_item(button, slot),
            SlotActionType::PickupAll => Ok(Self {
                click_type: ClickType::DoubleClick,
                slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            }),
        }
    }

    fn new_normal_click(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let slot = match slot {
            -999 => Slot::OutsideInventory,
            _ => {
                let slot = slot.try_into().unwrap_or(0);
                Slot::Normal(slot)
            }
        };
        let button = match button {
            0 => MouseClick::Left,
            1 => MouseClick::Right,
            _ => Err(InventoryError::InvalidPacket)?,
        };
        Ok(Self {
            click_type: ClickType::MouseClick(button),
            slot,
        })
    }

    fn new_shift_click(slot: i16) -> Result<Self, InventoryError> {
        Ok(Self {
            slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
            click_type: ClickType::ShiftClick,
        })
    }

    fn new_key_click(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let key = match button {
            0..9 => KeyClick::Slot(button.try_into().or(Err(InventoryError::InvalidSlot))?),
            40 => KeyClick::Offhand,
            _ => Err(InventoryError::InvalidSlot)?,
        };

        Ok(Self {
            click_type: ClickType::KeyClick(key),
            slot: Slot::Normal(slot.try_into().or(Err(InventoryError::InvalidSlot))?),
        })
    }

    fn new_drop_item(button: i8) -> Result<Self, InventoryError> {
        let drop_type = match button {
            0 => DropType::SingleItem,
            1 => DropType::FullStack,
            _ => Err(InventoryError::InvalidPacket)?,
        };
        Ok(Self {
            click_type: ClickType::DropType(drop_type),
            slot: Slot::OutsideInventory,
        })
    }

    fn new_drag_item(button: i8, slot: i16) -> Result<Self, InventoryError> {
        let state = match button {
            0 => MouseDragState::Start(MouseDragType::Left),
            4 => MouseDragState::Start(MouseDragType::Right),
            8 => MouseDragState::Start(MouseDragType::Middle),
            1 | 5 | 9 => {
                MouseDragState::AddSlot(slot.try_into().or(Err(InventoryError::InvalidSlot))?)
            }
            2 | 6 | 10 => MouseDragState::End,
            _ => Err(InventoryError::InvalidPacket)?,
        };
        Ok(Self {
            slot: match &state {
                MouseDragState::AddSlot(slot) => Slot::Normal(*slot),
                _ => Slot::OutsideInventory,
            },
            click_type: ClickType::MouseDrag { drag_state: state },
        })
    }
}

pub enum ClickType {
    MouseClick(MouseClick),
    ShiftClick,
    KeyClick(KeyClick),
    CreativePickItem,
    DropType(DropType),
    MouseDrag { drag_state: MouseDragState },
    DoubleClick,
}
#[derive(Debug, PartialEq, Eq)]
pub enum MouseClick {
    Left,
    Right,
}

pub enum KeyClick {
    Slot(u8),
    Offhand,
}
#[derive(Copy, Clone)]
pub enum Slot {
    Normal(usize),
    OutsideInventory,
}

pub enum DropType {
    SingleItem,
    FullStack,
}
#[derive(Debug, PartialEq)]
pub enum MouseDragType {
    Left,
    Right,
    Middle,
}
#[derive(PartialEq)]
pub enum MouseDragState {
    Start(MouseDragType),
    AddSlot(usize),
    End,
}

pub enum ItemChange {
    Remove { slot: usize },
    Add { slot: usize, item: ItemStack },
}
