use super::Command;

pub struct StopCommand {
    
}

impl<'a> Command<'a> for StopCommand {
    const NAME: &'static str = "stop";
    const DESCRIPTION: &'static str = "Stops the server";
    
    fn on_execute(sender: &mut super::CommandSender<'a>, command: String) {
        std::process::exit(0);
    }
    fn player_required() -> bool {
        true
    }
}