use async_trait::async_trait;

use crate::server::Server;

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg,
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
