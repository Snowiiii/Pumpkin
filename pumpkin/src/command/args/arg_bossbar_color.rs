use crate::command::args::{
    Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use crate::world::bossbar::BossbarColor;
use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

pub struct BossbarColorArgumentConsumer;

impl GetClientSideArgParser for BossbarColorArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        // Not sure if this is right...
        ProtoCmdArgParser::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for BossbarColorArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let color = match s {
            "blue" => Some(BossbarColor::Blue),
            "green" => Some(BossbarColor::Green),
            "pink" => Some(BossbarColor::Pink),
            "purple" => Some(BossbarColor::Purple),
            "red" => Some(BossbarColor::Red),
            "white" => Some(BossbarColor::White),
            "yellow" => Some(BossbarColor::Yellow),
            _ => None,
        };

        color.map(Arg::BossbarColor)
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let colors = ["blue", "green", "pink", "purple", "red", "white", "yellow"];
        let suggestions: Vec<CommandSuggestion> = colors
            .iter()
            .map(|color| CommandSuggestion::new((*color).to_string(), None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for BossbarColorArgumentConsumer {
    fn default_name(&self) -> String {
        "color".to_string()
    }
}

impl<'a> FindArg<'a> for BossbarColorArgumentConsumer {
    type Data = &'a BossbarColor;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::BossbarColor(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
