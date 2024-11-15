use crate::command::args::arg_sound::SoundArgumentConsumer;
use crate::command::args::arg_sound_category::SoundCategoryArgumentConsumer;
use crate::command::args::{
    arg_bounded_num::BoundedNumArgumentConsumer, arg_players::PlayersArgumentConsumer,
    arg_position_3d::Position3DArgumentConsumer, ConsumedArgs, FindArg, FindArgDefaultName,
};
use crate::command::tree_builder::argument_default_name;
use crate::command::{
    dispatcher::InvalidTreeError,
    tree::CommandTree,
    tree_builder::{argument, require},
    CommandExecutor, CommandSender,
};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

const NAMES: [&str; 1] = ["playsound"];
const DESCRIPTION: &str = "Plays a sound at a location.";

const ARG_SOUND: &str = "sound";
const ARG_SOURCE: &str = "source";
const ARG_TARGETS: &str = "targets";
const ARG_POSITION: &str = "pos";
const ARG_VOLUME: &str = "volume";
const ARG_PITCH: &str = "pitch";
const ARG_MIN_VOLUME: &str = "minVolume";

static VOLUME_CONSUMER: BoundedNumArgumentConsumer<f32> =
    BoundedNumArgumentConsumer::new().min(0.0).name(ARG_VOLUME);
static PITCH_CONSUMER: BoundedNumArgumentConsumer<f32> =
    BoundedNumArgumentConsumer::new().min(0.0).name(ARG_PITCH);
static MIN_VOLUME_CONSUMER: BoundedNumArgumentConsumer<f32> = BoundedNumArgumentConsumer::new()
    .max(1.0)
    .min(0.0)
    .name(ARG_MIN_VOLUME);

struct PlaySoundExecutor;

#[async_trait]
impl CommandExecutor for PlaySoundExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let sound = SoundArgumentConsumer::find_arg(args, ARG_SOUND)?;
        let source = SoundCategoryArgumentConsumer::find_arg(args, ARG_SOURCE)?;
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let position = match Position3DArgumentConsumer::find_arg(args, ARG_POSITION).ok() {
            Some(pos) => pos,
            None => {
                if let Some(player) = sender.as_player() {
                    player.living_entity.entity.pos.load()
                } else {
                    sender
                        .send_message(
                            TextComponent::text(
                                "You must specify a position if you are not a player",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(());
                }
            }
        };
        let Ok(volume) = VOLUME_CONSUMER
            .find_arg_default_name(args)
            .unwrap_or(Ok(1.0))
        else {
            sender
                .send_message(
                    TextComponent::text("Invalid volume, must be positive")
                        .color_named(NamedColor::Red),
                )
                .await;
            return Ok(());
        };
        let Ok(pitch) = PITCH_CONSUMER
            .find_arg_default_name(args)
            .unwrap_or(Ok(1.0))
        else {
            sender
                .send_message(
                    TextComponent::text("Invalid pitch, must be positive")
                        .color_named(NamedColor::Red),
                )
                .await;
            return Ok(());
        };
        // TODO: Use this value
        let Ok(_min_volume) = MIN_VOLUME_CONSUMER
            .find_arg_default_name(args)
            .unwrap_or(Ok(0.0))
        else {
            sender
                .send_message(
                    TextComponent::text("Invalid minimum volume, must be between 0.0 and 1.0")
                        .color_named(NamedColor::Red),
                )
                .await;
            return Ok(());
        };
        let worlds = targets
            .iter()
            .map(|target| target.living_entity.entity.world.clone())
            .collect::<Vec<_>>();

        for world in worlds {
            world
                .play_sound(sound, source, &position, volume, pitch)
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2).with_child(
            argument(ARG_SOUND, &SoundArgumentConsumer).with_child(
                argument(ARG_SOURCE, &SoundCategoryArgumentConsumer).with_child(
                    argument(ARG_TARGETS, &PlayersArgumentConsumer)
                        .with_child(
                            argument(ARG_POSITION, &Position3DArgumentConsumer)
                                .with_child(
                                    argument_default_name(&VOLUME_CONSUMER)
                                        .with_child(
                                            argument_default_name(&PITCH_CONSUMER)
                                                .with_child(
                                                    argument_default_name(&MIN_VOLUME_CONSUMER)
                                                        .execute(&PlaySoundExecutor),
                                                )
                                                .execute(&PlaySoundExecutor),
                                        )
                                        .execute(&PlaySoundExecutor),
                                )
                                .execute(&PlaySoundExecutor),
                        )
                        .execute(&PlaySoundExecutor),
                ),
            ),
        ),
    )
}
