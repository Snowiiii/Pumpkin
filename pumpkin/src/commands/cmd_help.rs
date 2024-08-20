use pumpkin_text::TextComponent;
use crate::commands::{CommandSender, DISPATCHER, dispatcher_init};
use crate::commands::dispatcher::{CommandDispatcher, InvalidTreeError};
use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::tree::{CommandTree, ConsumedArgs, RawArgs};
use crate::commands::tree_builder::argument;

pub(crate) const NAME: &str = "help";
pub(crate) const ALIAS: &str = "?";

const DESCRIPTION: &str = "Print a help message.";

const ARG_COMMAND: &str = "command";

fn consume_arg_command(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    let s = args.pop()?;

    let dispatcher = DISPATCHER;
    let dispatcher = dispatcher.get_or_init(dispatcher_init);

    if dispatcher.commands.contains_key(s) { Some(s.into()) }
    else { None }
}

fn parse_arg_command<'a>(consumed_args: &'a ConsumedArgs, dispatcher: &'a CommandDispatcher) -> Result<(&'a str, &'a CommandTree<'a>), InvalidTreeError> {
    let command_name = consumed_args.get(ARG_COMMAND)
        .ok_or(InvalidConsumptionError(None))?;

    if let Some(tree) = dispatcher.commands.get::<&str>(&command_name.as_str()) {
        Ok((command_name, tree))
    } else {
        Err(InvalidConsumptionError(Some(command_name.into())))
    }
}

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(DESCRIPTION).with_child(
        argument(ARG_COMMAND, consume_arg_command).execute(&|sender, args| {
            let dispatcher = DISPATCHER;
            let dispatcher = dispatcher.get_or_init(dispatcher_init);
            
            let (name, tree) = parse_arg_command(args, dispatcher)?;
            
            sender.send_message(
                TextComponent::text(
                    &format!("{} - {} Usage:{}", name, tree.description, tree.paths_formatted(name))
                )
            );
            
            Ok(())
        })
    ).execute(&|sender, _args| {
        let dispatcher = DISPATCHER;
        let dispatcher = dispatcher.get_or_init(dispatcher_init);

        for (name, tree) in &dispatcher.commands {
            sender.send_message(
                TextComponent::text(
                    &format!("{} - {} Usage:{}", name, tree.description, tree.paths_formatted(name))
                )
            );
        };

        Ok(())
    })
}