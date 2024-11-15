use crate::command::args::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg};
use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::sound::SoundCategory;
use std::str::FromStr;

pub(crate) struct SoundCategoryArgumentConsumer;

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

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::SoundCategory(data)) => Ok(*data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
