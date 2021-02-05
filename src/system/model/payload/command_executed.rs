use crate::command_system::CommandContext;

#[derive(Clone)]
crate struct CommandExecuted {
    crate command: &'static str,
    crate guild_name: String,
    crate context: CommandContext<'static>
}
