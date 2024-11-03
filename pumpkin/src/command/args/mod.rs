use std::{collections::HashMap, hash::Hash, sync::Arc};

use async_trait::async_trait;
use pumpkin_core::{
    math::{vector2::Vector2, vector3::Vector3},
    GameMode,
};

use crate::{entity::player::Player, server::Server};

use super::{
    tree::{CommandTree, RawArgs},
    CommandSender,
};

pub(crate) mod arg_bounded_num;
pub(crate) mod arg_command;
pub(crate) mod arg_entities;
pub(crate) mod arg_entity;
pub(crate) mod arg_gamemode;
pub(crate) mod arg_message;
pub(crate) mod arg_player;
pub(crate) mod arg_position_3d;
pub(crate) mod arg_postition_2d;
pub(crate) mod arg_rotation;
pub(crate) mod arg_simple;

/// see [`crate::commands::tree_builder::argument`]
/// Provide value or an Optional error message, If no Error message provided the default will be used
#[async_trait]
pub(crate) trait ArgumentConsumer: Sync {
    async fn consume<'a>(
        &self,
        sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>>;
}

pub(crate) trait DefaultNameArgConsumer: ArgumentConsumer {
    fn default_name(&self) -> &'static str;

    /// needed because trait upcasting is not stable
    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer;
}

#[derive(Clone)]
pub(crate) enum Arg<'a> {
    Entities(Vec<Arc<Player>>),
    Entity(Arc<Player>),
    Players(Vec<Arc<Player>>),
    Pos3D(Vector3<f64>),
    Pos2D(Vector2<f64>),
    Rotation(f32, f32),
    GameMode(GameMode),
    CommandTree(&'a CommandTree<'a>),
    Msg(String),
    F64(f64),
    F32(f32),
    I32(i32),
    #[allow(unused)]
    U32(u32),
    #[allow(unused)]
    Simple(String),
}

/// see [`crate::commands::tree_builder::argument`] and [`CommandTree::execute`]/[`crate::commands::tree_builder::NonLeafNodeBuilder::execute`]
pub(crate) type ConsumedArgs<'a> = HashMap<&'a str, Arg<'a>>;

pub(crate) trait GetCloned<K, V: Clone> {
    fn get_cloned(&self, key: &K) -> Option<V>;
}

impl<K: Eq + Hash, V: Clone> GetCloned<K, V> for HashMap<K, V> {
    fn get_cloned(&self, key: &K) -> Option<V> {
        self.get(key).cloned()
    }
}

#[macro_export]
macro_rules! get_parsed_arg {
    ($args:ident, $name:expr, $p:pat, $out:expr) => {
        match $args.get(&$name) {
            Some($p) => Ok($out),
            _ => Err(
                $crate::command::dispatcher::InvalidTreeError::InvalidConsumptionError(Some(
                    $name.into(),
                )),
            ),
        }
    };
}

#[macro_export]
macro_rules! get_parsed_arg_default {
    ($args:ident, $consumer:expr, $p:pat, $out:expr) => {{
        use $crate::command::args::DefaultNameArgConsumer;
        let name = $consumer.default_name();
        match $args.get(name) {
            Some($p) => Ok($out),
            _ => Err(
                $crate::command::dispatcher::InvalidTreeError::InvalidConsumptionError(Some(
                    name.into(),
                )),
            ),
        }
    }};
}
