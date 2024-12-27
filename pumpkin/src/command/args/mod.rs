use std::{collections::HashMap, hash::Hash, sync::Arc};

use arg_bounded_num::{NotInBounds, Number};
use async_trait::async_trait;
use pumpkin_core::{
    math::{position::WorldPosition, vector2::Vector2, vector3::Vector3},
    GameMode,
};
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use super::{
    dispatcher::CommandError,
    tree::{CommandTree, RawArgs},
    CommandSender,
};
use crate::world::bossbar::{BossbarColor, BossbarDivisions};
use crate::{entity::player::Player, server::Server};

pub mod arg_block;
pub mod arg_bool;
pub mod arg_bossbar_color;
pub mod arg_bossbar_style;
pub mod arg_bounded_num;
pub mod arg_command;
pub mod arg_entities;
pub mod arg_entity;
pub mod arg_gamemode;
pub mod arg_item;
pub mod arg_message;
pub mod arg_players;
pub mod arg_position_2d;
pub mod arg_position_3d;
pub mod arg_position_block;
pub mod arg_resource_location;
pub mod arg_rotation;
pub mod arg_simple;
mod coordinate;

/// see [`crate::commands::tree_builder::argument`]
#[async_trait]
pub trait ArgumentConsumer: Sync + GetClientSideArgParser {
    async fn consume<'a>(
        &'a self,
        sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg>;

    /// Used for tab completion (but only if argument suggestion type is "minecraft:ask_server"!).
    ///
    /// NOTE: This is called after this consumer's [`ArgumentConsumer::consume`] method returned None, so if args is used here, make sure [`ArgumentConsumer::consume`] never returns None after mutating args.
    async fn suggest<'a>(
        &'a self,
        sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError>;
}

pub trait GetClientSideArgParser {
    /// Return the parser the client should use while typing a command in chat.
    fn get_client_side_parser(&self) -> ProtoCmdArgParser;
    /// Usually this should return None. This can be used to force suggestions to be processed on serverside.
    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType>;
}

pub trait DefaultNameArgConsumer: ArgumentConsumer {
    fn default_name(&self) -> String;
}

#[derive(Clone)]
pub enum Arg<'a> {
    Entities(Vec<Arc<Player>>),
    Entity(Arc<Player>),
    Players(Vec<Arc<Player>>),
    BlockPos(WorldPosition),
    Pos3D(Vector3<f64>),
    Pos2D(Vector2<f64>),
    Rotation(f32, f32),
    GameMode(GameMode),
    CommandTree(CommandTree),
    Item(&'a str),
    ResourceLocation(&'a str),
    Block(&'a str),
    BossbarColor(BossbarColor),
    BossbarStyle(BossbarDivisions),
    Msg(String),
    Num(Result<Number, NotInBounds>),
    Bool(bool),
    #[allow(unused)]
    Simple(&'a str),
}

/// see [`crate::commands::tree_builder::argument`] and [`CommandTree::execute`]/[`crate::commands::tree_builder::NonLeafNodeBuilder::execute`]
pub type ConsumedArgs<'a> = HashMap<&'a str, Arg<'a>>;

pub(crate) trait GetCloned<K, V: Clone> {
    fn get_cloned(&self, key: &K) -> Option<V>;
}

impl<K: Eq + Hash, V: Clone> GetCloned<K, V> for HashMap<K, V> {
    fn get_cloned(&self, key: &K) -> Option<V> {
        self.get(key).cloned()
    }
}

pub trait FindArg<'a> {
    type Data;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError>;
}

pub(crate) trait FindArgDefaultName<'a, T> {
    fn find_arg_default_name(&self, args: &'a ConsumedArgs) -> Result<T, CommandError>;
}

impl<'a, T, C: FindArg<'a, Data = T> + DefaultNameArgConsumer> FindArgDefaultName<'a, T> for C {
    fn find_arg_default_name(&self, args: &'a ConsumedArgs) -> Result<T, CommandError> {
        C::find_arg(args, &self.default_name())
    }
}

pub(crate) trait SplitSingleWhitespaceIncludingEmptyParts<'a> {
    /// Splits a string at every single unicode whitespace. Therefore the returned iterator sometimes contains empty strings. This is useful for command suggestions.
    ///
    /// Note: Vanilla does this only for command suggestions, for execution consecutive whitespaces are treated as one.
    fn split_single_whitespace_including_empty_parts(self) -> impl Iterator<Item = &'a str>;
}

impl<'a> SplitSingleWhitespaceIncludingEmptyParts<'a> for &'a str {
    fn split_single_whitespace_including_empty_parts(self) -> impl Iterator<Item = &'a str> {
        SplitSingleWhitespaceIncludingEmptyPartsIter {
            s: self,
            pos: 0,
            chars_iter: self.char_indices(),
            is_complete: false,
        }
    }
}

struct SplitSingleWhitespaceIncludingEmptyPartsIter<'a, T: Iterator<Item = (usize, char)>> {
    s: &'a str,
    pos: usize,
    chars_iter: T,
    is_complete: bool,
}

impl<'a, T: DoubleEndedIterator<Item = (usize, char)>> Iterator
    for SplitSingleWhitespaceIncludingEmptyPartsIter<'a, T>
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_complete {
            return None;
        }

        let start = self.pos;

        loop {
            match self.chars_iter.next() {
                Some((i, c)) if c.is_whitespace() => {
                    let whitespace_len = c.len_utf8();
                    self.pos = i + whitespace_len;
                    return Some(&self.s[start..i]);
                }
                Some(_) => {}
                None => {
                    self.is_complete = true;
                    return Some(&self.s[start..self.pos]);
                }
            };
        }
    }
}
