use thiserror::Error;

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("Unable to lock")]
    LockError,
    #[error("Invalid slot")]
    InvalidSlot,
    #[error("Player '{0}' tried to interact with a closed container")]
    ClosedContainerInteract(i32),
    #[error("Multiple players dragging in a container at once")]
    MultiplePlayersDragging,
    #[error("Out of order dragging")]
    OutOfOrderDragging,
    #[error("Invalid inventory packet")]
    InvalidPacket,
    #[error("Player does not have enough permissions")]
    PermissionError,
}

impl InventoryError {
    pub fn should_kick(&self) -> bool {
        match self {
            InventoryError::InvalidSlot
            | InventoryError::ClosedContainerInteract(..)
            | InventoryError::InvalidPacket
            | InventoryError::PermissionError => true,
            InventoryError::LockError
            | InventoryError::OutOfOrderDragging
            | InventoryError::MultiplePlayersDragging => false,
        }
    }
}
