use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior
};

use crate::{
    command::{
        dispatcher::InvalidTreeError, tree::{CommandTree, RawArgs}, CommandSender
    },
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub(crate) struct ClientSideArgParserTester<'a>(pub ProtoCmdArgParser<'a>);

impl<'a> GetClientSideArgParser for ClientSideArgParserTester<'a> {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        self.0.clone()
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl<'b> ArgumentConsumer for ClientSideArgParserTester<'b> {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        None
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {
        dbg!(input);
        Ok(None)
    }
}

impl<'a> DefaultNameArgConsumer for ClientSideArgParserTester<'a> {
    fn default_name(&self) -> &'static str {
        "ClientSideArgParserArgumentConsumer"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        self
    }
}

impl<'a> FindArg<'a> for ClientSideArgParserTester<'a> {
    type Data = ();

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        Ok(())
    }
}
