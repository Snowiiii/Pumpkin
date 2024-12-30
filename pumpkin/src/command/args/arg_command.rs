use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior,
};

use crate::{
    command::{
        args::SplitSingleWhitespaceIncludingEmptyParts,
        dispatcher::CommandError,
        tree::{CommandTree, RawArgs},
        CommandSender,
    },
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct CommandTreeArgumentConsumer;

impl GetClientSideArgParser for CommandTreeArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for CommandTreeArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let dispatcher = server.command_dispatcher.read().await;
        dispatcher
            .get_tree(s)
            .map_or_else(|_| None, |tree| Some(Arg::CommandTree(tree.clone())))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let Some(input) = input.split_single_whitespace_including_empty_parts().last() else {
            return Ok(None);
        };

        let dispatcher = server.command_dispatcher.read().await;
        let suggestions = dispatcher
            .commands
            .keys()
            .filter(|suggestion| suggestion.starts_with(input))
            .map(|suggestion| CommandSuggestion::new(suggestion.to_string(), None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for CommandTreeArgumentConsumer {
    fn default_name(&self) -> String {
        "cmd".to_string()
    }
}

impl<'a> FindArg<'a> for CommandTreeArgumentConsumer {
    type Data = &'a CommandTree;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::CommandTree(tree)) => Ok(tree),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
