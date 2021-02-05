use std::borrow::Cow;

use crate::command_system::{
    events::{
        events::{
            SystemEvent,
            CommandEvents
        },
        listener::Listeners
    },
    parser::{
        CommandParser,
        CommandParserConfiguration,
    },
    Command
};

#[derive(Clone)]
crate struct CommandFramework<'a> {
    command_parser_config: CommandParserConfiguration<'a>,
    listeners: Listeners<SystemEvent>
}

impl<'a> CommandFramework<'a> {
    crate fn new() -> Self {
        Self {
            command_parser_config: CommandParserConfiguration::default(),
            listeners: Listeners::default()
        }
    }

    crate fn command<T>(mut self, command: T, case_sensitive: bool, fully_qualified_name: bool, enable_aliases: bool) -> Self
    where T: Command {
        self
            .command_parser_config
            .add_command(
                if fully_qualified_name {
                    command.fully_qualified_name()
                } else {
                    command.name()
                }, case_sensitive);

        if enable_aliases {
            for alias in command.aliases() {
                self.command_parser_config.add_command(alias, case_sensitive);
            }
        }

        self
    }

    crate fn command_prefix(mut self, prefix: impl Into<Cow<'a, str>>) -> Self {
        self.command_parser_config.add_prefix(prefix);

        self
    }

    crate fn build_parser(self) -> CommandParser<'a> {
        CommandParser::new(self.command_parser_config)
    }

    crate fn listeners(self) -> Listeners<SystemEvent> {
        self.listeners
    }

    crate fn events(self) -> CommandEvents {
        let rx = self.listeners.add();

        CommandEvents::new(rx)
    }
}
