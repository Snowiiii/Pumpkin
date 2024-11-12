use std::{collections::HashMap, hash::Hash, sync::Arc};

use arg_bounded_num::{NotInBounds, Number};
use async_trait::async_trait;
use pumpkin_core::{
    math::{vector2::Vector2, vector3::Vector3},
    GameMode,
};

use crate::{entity::player::Player, server::Server};

use super::{
    dispatcher::CommandError,
    tree::{CommandTree, RawArgs},
    CommandSender,
};

pub(crate) mod arg_bounded_num;
pub(crate) mod arg_command;
pub(crate) mod arg_entities;
pub(crate) mod arg_entity;
pub(crate) mod arg_gamemode;
pub(crate) mod arg_item;
pub(crate) mod arg_message;
pub(crate) mod arg_players;
pub(crate) mod arg_position_2d;
pub(crate) mod arg_position_3d;
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
    Item(String),
    Msg(String),
    Num(Result<Number, NotInBounds>),
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

pub(crate) trait FindArg<'a> {
    type Data;

    fn find_arg(args: &'a ConsumedArgs, name: &'a str) -> Result<Self::Data, CommandError>;
}

pub(crate) trait FindArgDefaultName<'a, T> {
    fn find_arg_default_name(&self, args: &'a ConsumedArgs) -> Result<T, CommandError>;
}

impl<'a, T, C: FindArg<'a, Data = T> + DefaultNameArgConsumer> FindArgDefaultName<'a, T> for C {
    fn find_arg_default_name(&self, args: &'a ConsumedArgs) -> Result<T, CommandError> {
        C::find_arg(args, self.default_name())
    }
}
