use log::log;
use pumpkin_inventory::InventoryError;
use pumpkin_protocol::bytebuf::DeserializerError;
use std::fmt::Display;

use crate::{client::player_packet::PlayerError, world::WorldError};

pub trait PumpkinError: Send + std::error::Error + Display {
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
        use InventoryError::{
            ClosedContainerInteract, InvalidPacket, InvalidSlot, LockError,
            MultiplePlayersDragging, OutOfOrderDragging, PermissionError,
        };
        match self {
            InvalidSlot | ClosedContainerInteract(..) | InvalidPacket | PermissionError => true,
            LockError | OutOfOrderDragging | MultiplePlayersDragging => false,
        }
    }
    fn severity(&self) -> log::Level {
        use InventoryError::{
            ClosedContainerInteract, InvalidPacket, InvalidSlot, LockError,
            MultiplePlayersDragging, OutOfOrderDragging, PermissionError,
        };
        match self {
            LockError
            | InvalidSlot
            | ClosedContainerInteract(..)
            | InvalidPacket
            | PermissionError => log::Level::Error,
            OutOfOrderDragging => log::Level::Info,
            MultiplePlayersDragging => log::Level::Warn,
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

impl PumpkinError for WorldError {
    fn is_kick(&self) -> bool {
        false
    }

    fn severity(&self) -> log::Level {
        log::Level::Warn
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}

impl PumpkinError for PlayerError {
    fn is_kick(&self) -> bool {
        match self {
            Self::BlockOutOfReach => false,
            Self::InvalidBlockFace => true,
        }
    }

    fn severity(&self) -> log::Level {
        match self {
            Self::BlockOutOfReach | Self::InvalidBlockFace => log::Level::Warn,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::BlockOutOfReach => None,
            Self::InvalidBlockFace => Some("Invalid block face".into()),
        }
    }
}
