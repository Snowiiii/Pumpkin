use async_trait::async_trait;

use crate::{command::dispatcher::InvalidTreeError, server::Server};

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg, FindArg,
};

/// Should never be a permanent solution
#[allow(unused)]
pub(crate) struct SimpleArgConsumer;

#[async_trait]
impl ArgumentConsumer for SimpleArgConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        Some(Arg::Simple(args.pop()?.to_string()))
    }
}

impl<'a> FindArg<'a> for SimpleArgConsumer {

    type Data = &'a str;

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Simple(data)) => Ok(data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(name.to_string())))
        }
    }
}
