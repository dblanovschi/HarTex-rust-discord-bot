use std::{
    future::Future,
    pin::Pin
};

use compound_duration::format_dhms;

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

crate struct UptimeCommand;

impl Command for UptimeCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("uptime")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_uptime_command(ctx))
    }
}

async fn general_uptime_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let elapsed = ctx.stopwatch.elapsed_seconds();
    let embed = EmbedBuilder::new()
        .title("Bot Uptime")?
        .color(0x03_BE_FC)?
        .field(EmbedFieldBuilder::new("Current Uptime", format_dhms(elapsed))?)
        .build()?;

    ctx.http_client.create_message(ctx.message.channel_id).reply(ctx.message.id).embed(embed)?.allowed_mentions()
        .replied_user(false).build().await?;

    Ok(())
}
