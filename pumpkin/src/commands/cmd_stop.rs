use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::commands::tree::CommandTree;
use crate::commands::tree_builder::require;

use super::RunFunctionType;

const NAMES: [&str; 1] = ["stop"];

const DESCRIPTION: &str = "Stop the server.";

struct StopExecutor {}

#[async_trait]
impl RunFunctionType for StopExecutor {
    async fn execute(
        &self,
        sender: &mut super::CommandSender,
        _server: &crate::server::Server,
        _args: &super::tree::ConsumedArgs,
    ) -> Result<(), super::dispatcher::InvalidTreeError> {
        sender
            .send_message(TextComponent::text("Stopping Server").color_named(NamedColor::Red))
            .await;

        // TODO: Gracefully stop
        std::process::exit(0)
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(require(&|sender| sender.permission_lvl() >= 4).execute(&StopExecutor {}))
}
