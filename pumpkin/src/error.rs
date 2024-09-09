use log::log;
use pumpkin_inventory::InventoryError;
use pumpkin_protocol::bytebuf::DeserializerError;
use std::fmt::Display;

pub trait PumpkinError: std::error::Error + Display {
    fn is_kick(&self) -> bool;

    fn log(&self) {
        log!(self.severity(), "{}", self.to_string());
    }

    fn severity(&self) -> log::Level;

    fn client_kick_reason(&self) -> Option<String>;
}

impl<ErrorType: PumpkinError + 'static> From<ErrorType> for Box<dyn PumpkinError> {
    fn from(error: ErrorType) -> Self {
        Box::new(error)
    }
}
impl PumpkinError for InventoryError {
    fn is_kick(&self) -> bool {
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
    fn severity(&self) -> log::Level {
        match self {
            InventoryError::LockError
            | InventoryError::InvalidSlot
            | InventoryError::ClosedContainerInteract(..)
            | InventoryError::InvalidPacket
            | InventoryError::PermissionError => log::Level::Error,
            InventoryError::OutOfOrderDragging => log::Level::Info,
            InventoryError::MultiplePlayersDragging => log::Level::Warn,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}

impl PumpkinError for DeserializerError {
    fn is_kick(&self) -> bool {
        true
    }

    fn severity(&self) -> log::Level {
        log::Level::Error
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}
