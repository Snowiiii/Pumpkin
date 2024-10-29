use async_trait::async_trait;

use crate::server::Server;

use super::{
    tree::{ArgumentConsumer, RawArgs},
    CommandSender,
};

/// Should never be a permanent solution
pub(crate) struct SimpleArgConsumer;

#[async_trait]
impl ArgumentConsumer for SimpleArgConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        args.pop().ok_or(None).map(ToString::to_string)
    }
}
