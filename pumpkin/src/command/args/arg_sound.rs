use crate::command::args::{
    Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::sound::{Sound, SOUNDS};
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

pub(crate) struct SoundArgumentConsumer;

impl GetClientSideArgParser for SoundArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AvailableSounds)
    }
}

#[async_trait]
impl ArgumentConsumer for SoundArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let name = args.pop()?;
        SOUNDS.get(name).map(|id| {
            Arg::Sound(Sound {
                name: name.to_string(),
                id: *id,
            })
        })
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for SoundArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "sound"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &SoundArgumentConsumer
    }
}

impl<'a> FindArg<'a> for SoundArgumentConsumer {
    type Data = Sound;

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Sound(sound)) => Ok(sound.clone()),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
