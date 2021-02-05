use std::{
    future::Future,
    pin::Pin
};

use sysinfo::{
    ProcessExt,
    SystemExt
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

crate struct AboutCommand;

impl Command for AboutCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("about")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_about_command(ctx))
    }
}

async fn general_about_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let message_id = ctx.message.id;
    let channel_id = ctx.message.channel_id;
    let guilds_count = ctx.http_client.current_user_guilds().await?.len();

    let system_information = sysinfo::System::new_all();

    let hartex_process = system_information.get_processes().iter().find(|(_, process)| {
        process.name() == "hartex_rewrite.exe"
    });

    let embed = EmbedBuilder::new()
        .title("About HarTex Beta")?
        .description(
            "HarTex Beta is the in-development version of HarTex, built and optimized for moderation and administration"
                .to_owned() + " for Discord guilds.")?
        .color(0x03_BE_FC)?
        .field(EmbedFieldBuilder::new("Whitelisted Guilds", format!("{}", guilds_count))?)
        .field(EmbedFieldBuilder::new("Hardware Usage",
                                      format!("RAM Usage: `{}`MB\nCPU Usage: `{}`%", match hartex_process {
            Some((_, process)) => {
                (process.memory() / 1000).to_string()
            },
            None => "unavailable".to_string()
        },
        match hartex_process {
            Some((_, process)) => {
                process.cpu_usage().to_string()
            },
            None => "unavailable".to_string()
        }))?)
        .build()?;

    ctx.http_client.create_message(channel_id).embed(embed)?.reply(message_id).allowed_mentions()
        .replied_user(false).build().await?;

    Ok(())
}
