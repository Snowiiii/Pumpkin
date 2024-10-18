use crate::commands::tree::CommandTree;

const NAMES: [&str; 1] = ["pumpkin"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&|sender, _, _| {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");

        // sender.send_message(TextComponent::text(
        //     &format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")
        // ).color_named(NamedColor::Green));

        Ok(())
    })
}
