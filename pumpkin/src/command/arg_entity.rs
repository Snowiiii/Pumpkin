use std::sync::Arc;

use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::arg_player::{parse_arg_player, PlayerArgumentConsumer};
use super::tree::ArgumentConsumer;

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
///
/// For selecting a single entity, eg. using @s, a player name or entity uuid
pub(crate) struct EntityArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for EntityArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        PlayerArgumentConsumer.consume(src, server, args).await
    }
}

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
pub(crate) async fn parse_arg_entity<'a>(
    src: &mut CommandSender<'a>,
    server: &Server,
    arg_name: &str,
    consumed_args: &ConsumedArgs<'a>,
) -> Result<Arc<crate::entity::player::Player>, InvalidTreeError> {
    parse_arg_player(src, server, arg_name, consumed_args).await
}
