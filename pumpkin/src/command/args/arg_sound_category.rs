use crate::command::args::{
    Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::sound::{SoundCategory, SOUND_CATEGORIES};
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior,
};
use std::str::FromStr;

pub(crate) struct SoundCategoryArgumentConsumer;

impl GetClientSideArgParser for SoundCategoryArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for SoundCategoryArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        SoundCategory::from_str(args.pop()?)
            .ok()
            .map(Arg::SoundCategory)
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, CommandError> {
        let sound_category_names = SOUND_CATEGORIES.keys();
        let args = input.split_whitespace().collect::<Vec<_>>();
        let Some(input) = args.get(2) else {
            return Ok(Some(
                sound_category_names
                    .map(|name| CommandSuggestion::new(name, None))
                    .collect(),
            ));
        };
        Ok(Some(
            sound_category_names
                .filter(|name| name.starts_with(input))
                .map(|name| CommandSuggestion::new(name, None))
                .collect(),
        ))
    }
}

impl DefaultNameArgConsumer for SoundCategoryArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "sound"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &SoundCategoryArgumentConsumer
    }
}

impl<'a> FindArg<'a> for SoundCategoryArgumentConsumer {
    type Data = SoundCategory;

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::SoundCategory(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
