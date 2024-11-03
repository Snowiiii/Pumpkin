use async_trait::async_trait;

use crate::{
    command::{tree::RawArgs, CommandSender},
    server::Server,
};

use super::{Arg, ArgumentConsumer};

pub(crate) struct CommandTreeArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for CommandTreeArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let dispatcher = &server.command_dispatcher;
        return match dispatcher.get_tree(s) {
            Ok(tree) => Some(Arg::CommandTree(tree)),
            Err(_) => None,
        };
    }
}
