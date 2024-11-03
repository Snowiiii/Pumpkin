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

/// This macro can be used to easily get the correct argument from [`ConsumedArgs`].
///
/// See [`get_parsed_arg_default`] if you are using [`crate::command::tree_builder::argument_default`].
///
/// # Example
///
/// `let distance: f64 = get_parsed_arg!(args, ARG_DISTANCE, Arg::F64(v), *v)?;`.
///
/// - `args` is the [`ConsumedArgs`] which is passed to executors as a parameter.
///
/// - `ARGS_DISTANCE` is the name of the argument, which must be the same as the one used in `init_command_tree`
///
/// - `Arg::F64(v)` is a pattern used for patten matching. The variant of the enum [`Arg`] that should be matched here is determined by the consumer that was used.
///
/// - `*v` is the part of the pattern that should be returned. Operations like cloning or dereferencing can be done here, too.
///
/// A Result<T, [`super::dispatcher::InvalidTreeError`]> is returned, where T is determined by the last macro parameter.
/// The returned Result enum has the same error type as the Result command executors return, so the `?` operator can be used.
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

/// This macro can be used to easily get the correct argument from [`ConsumedArgs`], using the default name defined by the [`ArgumentConsumer`].
///
/// Use this only when you're also using [`crate::command::tree_builder::argument_default`] and don't need two arguments with the same [`ArgumentConsumer`].
/// Otherwise use [`crate::command::tree_builder::argument`] and [`get_parsed_arg`].
///
/// # Example
///
/// `let Vector2 { x, z } = get_parsed_arg_default!(args, Position2DArgumentConsumer, Arg::Pos2D(vec), *vec)?;`
///
/// - `args` is the [`ConsumedArgs`] which is passed to executors as a parameter.
///
/// - `Position2DArgumentConsumer` is the [`ArgumentConsumer`], which must be the same as the one used in the [`crate::command::tree_builder::argument_default`] method in `init_command_tree`
///
/// - `Arg::Pos2D(vec)` is a pattern used for patten matching. The variant of the enum [`Arg`] that should be matched here is determined by the consumer that was used.
///
/// - `*vec` is the part of the pattern that should be returned. Operations like cloning or dereferencing can be done here, too.
///
/// A Result<T, [`super::dispatcher::InvalidTreeError`]> is returned, where T is determined by the last macro parameter.
/// The returned Result enum has the same error type as the Result command executors return, so the `?` operator can be used.
#[macro_export]
macro_rules! get_parsed_arg_with_default_name {
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
