use async_trait::async_trait;

use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::arg_player::PlayersArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer};

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
///
/// For selecting zero, one or multiple entities, eg. using @s, a player name, @a or @e
pub(crate) struct EntitiesArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for EntitiesArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        match PlayersArgumentConsumer.consume(src, server, args).await {
            Some(Arg::Players(p)) => Some(Arg::Entities(p)),
            _ => None,
        }
    }
}

impl DefaultNameArgConsumer for EntitiesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "targets"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &EntitiesArgumentConsumer
    }
}
