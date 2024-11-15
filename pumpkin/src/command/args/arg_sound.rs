use crate::command::args::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg};
use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::sound::SOUNDS;

pub(crate) struct SoundArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for SoundArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        SOUNDS.get(args.pop()?).map(|sound| Arg::Sound(*sound))
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
    type Data = u16;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Sound(data)) => Ok(*data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
