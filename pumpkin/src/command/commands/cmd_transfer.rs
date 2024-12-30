use async_trait::async_trait;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CTransfer;
use pumpkin_protocol::codec::var_int::VarInt;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::arg_players::PlayersArgumentConsumer;
use crate::command::args::arg_simple::SimpleArgConsumer;
use crate::command::args::{Arg, FindArgDefaultName};
use crate::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use crate::command::tree_builder::{argument, argument_default_name, require};
use crate::command::{
    args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
};

const NAMES: [&str; 1] = ["transfer"];

const DESCRIPTION: &str = "Triggers a transfer of a player to another server.";

const ARG_HOSTNAME: &str = "hostname";

const ARG_PLAYERS: &str = "players";

fn port_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("port")
        .min(1)
        .max(65535)
}

struct TransferTargetSelf;

#[async_trait]
impl CommandExecutor for TransferTargetSelf {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
            return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
        };

        let port = match port_consumer().find_arg_default_name(args) {
            Err(_) => 25565,
            Ok(Ok(count)) => count,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Port must be between 1 and 65535.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        if let CommandSender::Player(player) = sender {
            let name = &player.gameprofile.name;
            log::info!("[{name}: Transferring {name} to {hostname}:{port}]");
            player
                .client
                .send_packet(&CTransfer::new(hostname, &VarInt(port)))
                .await;
            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

struct TransferTargetPlayer;

#[async_trait]
impl CommandExecutor for TransferTargetPlayer {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
            return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
        };

        let port = match port_consumer().find_arg_default_name(args) {
            Err(_) => 25565,
            Ok(Ok(count)) => count,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Port must be between 1 and 65535.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        let Some(Arg::Players(players)) = args.get(ARG_PLAYERS) else {
            return Err(InvalidConsumption(Some(ARG_PLAYERS.into())));
        };

        for p in players {
            p.client
                .send_packet(&CTransfer::new(hostname, &VarInt(port)))
                .await;
            log::info!(
                "[{sender}: Transferring {} to {hostname}:{port}]",
                p.gameprofile.name
            );
        }

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_HOSTNAME, SimpleArgConsumer)
            .with_child(require(|sender| sender.is_player()).execute(TransferTargetSelf))
            .with_child(
                argument_default_name(port_consumer())
                    .with_child(require(|sender| sender.is_player()).execute(TransferTargetSelf))
                    .with_child(
                        argument(ARG_PLAYERS, PlayersArgumentConsumer)
                            .execute(TransferTargetPlayer),
                    ),
            ),
    )
}
