use async_trait::async_trait;

use crate::{command::dispatcher::InvalidTreeError, server::Server};

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg, DefaultNameArgConsumer, FindArg,
};

pub(crate) struct ItemArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for ItemArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let name = if s.starts_with("minecraft:") {
            s.to_string()
        } else {
            format!("minecraft:{s}")
        };

        // todo: get an actual item
        Some(Arg::Item(name))
    }
}

impl DefaultNameArgConsumer for ItemArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "item"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        self
    }
}

impl<'a> FindArg<'a> for ItemArgumentConsumer {
    type Data = &'a str;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Item(name)) => Ok(name),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
