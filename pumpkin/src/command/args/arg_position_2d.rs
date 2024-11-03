use async_trait::async_trait;
use pumpkin_core::math::vector2::Vector2;

use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer};

/// x and z coordinates only
///
/// todo: implememnt ~ ^ notations
pub(crate) struct Position2DArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for Position2DArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let x = args.pop()?.parse::<f64>().ok()?;
        let z = args.pop()?.parse::<f64>().ok()?;

        Some(Arg::Pos2D(Vector2::new(x, z)))
    }
}

impl DefaultNameArgConsumer for Position2DArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "pos2d"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &Position2DArgumentConsumer
    }
}
