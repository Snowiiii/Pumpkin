use async_trait::async_trait;

use crate::{
    command::{
        dispatcher::CommandError,
        tree::{CommandTree, RawArgs},
        CommandSender,
    },
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg};

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

impl DefaultNameArgConsumer for CommandTreeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "cmd"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &CommandTreeArgumentConsumer
    }
}

impl<'a> FindArg<'a> for CommandTreeArgumentConsumer {
    type Data = &'a CommandTree<'a>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::CommandTree(tree)) => Ok(tree),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
