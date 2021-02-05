use std::{
    future::Future,
    pin::Pin,
    process::Command as StdCommand
};

use twilight_cache_inmemory::{
    InMemoryCache
};

use twilight_embed_builder::{
    EmbedBuilder,
    EmbedFieldBuilder
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
};

use crate::system::{
    SystemResult,
};

use crate::utilities::constants::hartex_version;

crate struct BotinfoCommand;

impl Command for BotinfoCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("bot-info")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments,
                                            _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_botinfo_command(ctx))
    }
}

async fn general_botinfo_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let rust_version = String::from_utf8(StdCommand::new("rustc")
        .arg("--version").output().unwrap().stdout).unwrap();

    let embed = EmbedBuilder::new()
        .color(0x03_BE_FC)?
        .title("About HarTex")?
        .description("Information about me.")?
        .field(EmbedFieldBuilder::new("Bot Version", hartex_version())?)
        .field(EmbedFieldBuilder::new("Programming Language Version: Rust", rust_version)?)
        .build()?;
    
    ctx.http_client.create_message(channel_id).reply(ctx.message.id).allowed_mentions().replied_user(false)
        .build().embed(embed)?.await?;

    Ok(())
}
