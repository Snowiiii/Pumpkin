use crate::command::args::{
    Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use crate::world::bossbar::BossbarDivisions;
use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

pub struct BossbarStyleArgumentConsumer;

impl GetClientSideArgParser for BossbarStyleArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        // Not sure if this is right...
        ProtoCmdArgParser::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for BossbarStyleArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let style = match s {
            "notched_10" => Some(BossbarDivisions::Notches10),
            "notched_12" => Some(BossbarDivisions::Notches12),
            "notched_20" => Some(BossbarDivisions::Notches20),
            "notched_6" => Some(BossbarDivisions::Notches6),
            "progress" => Some(BossbarDivisions::NoDivision),
            _ => None,
        };

        style.map(Arg::BossbarStyle)
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let styles = [
            "notched_10",
            "notched_12",
            "notched_20",
            "notched_6",
            "progress",
        ];
        let suggestions: Vec<CommandSuggestion> = styles
            .iter()
            .map(|style| CommandSuggestion::new((*style).to_string(), None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for BossbarStyleArgumentConsumer {
    fn default_name(&self) -> String {
        "style".to_string()
    }
}

impl<'a> FindArg<'a> for BossbarStyleArgumentConsumer {
    type Data = &'a BossbarDivisions;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::BossbarStyle(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
