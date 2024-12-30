use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};
use pumpkin_world::item::item_registry::{self, Item};

use crate::{command::dispatcher::CommandError, server::Server};

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

pub struct ItemArgumentConsumer;

impl GetClientSideArgParser for ItemArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Resource { identifier: "item" }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for ItemArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        // todo: get an actual item
        Some(Arg::Item(args.pop()?))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for ItemArgumentConsumer {
    fn default_name(&self) -> String {
        "item".to_string()
    }
}

impl<'a> FindArg<'a> for ItemArgumentConsumer {
    type Data = (&'a str, &'a Item);

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Item(name)) => item_registry::get_item(name).map_or_else(
                || {
                    Err(CommandError::GeneralCommandIssue(format!(
                        "Item {name} does not exist."
                    )))
                },
                |item| Ok((*name, item)),
            ),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
