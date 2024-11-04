use async_trait::async_trait;

use crate::{command::dispatcher::InvalidTreeError, server::Server};

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg, DefaultNameArgConsumer, FindArg,
};

/// Consumes all remaining words/args. Does not consume if there is no word.
pub(crate) struct MsgArgConsumer;

#[async_trait]
impl ArgumentConsumer for MsgArgConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let mut msg = args.pop()?.to_string();

        while let Some(word) = args.pop() {
            msg.push(' ');
            msg.push_str(word);
        }

        Some(Arg::Msg(msg))
    }
}

impl DefaultNameArgConsumer for MsgArgConsumer {
    fn default_name(&self) -> &'static str {
        "msg"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &MsgArgConsumer
    }
}

impl<'a> FindArg<'a> for MsgArgConsumer {
    type Data = &'a str;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Msg(data)) => Ok(data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
