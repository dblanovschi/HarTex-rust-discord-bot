use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache
};
use twilight_embed_builder::{
    EmbedBuilder,
    EmbedFieldBuilder,
    EmbedFooterBuilder
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
};

use crate::system::SystemResult;

crate struct HelpCommand;

impl Command for HelpCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("help")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments,
                                            _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_help_command(ctx))
    }
}

async fn general_help_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let embed = EmbedBuilder::new()
        .title("HarTex - Moderation Commands Reference")?
        .description(
            "The following list are the common commands used by the moderators. A more detailed list can be found at the"
                .to_owned()
                + " documentation website: SOON:tm:")?
        .field(EmbedFieldBuilder::new(
            "warn <User ID> [Reason]", "Adds an infraction to a user with the type `Warning`.")?)
        .footer(EmbedFooterBuilder::new(
            "For parameters wrapped in `<>`, it can either be a user mention or a User ID. For parameters wrapped"
                .to_owned() + " in `[]`, it is optional.")?)
        .color(0x03_BE_FC)?
        .build()?;

    ctx.http_client.create_message(channel_id).reply(ctx.message.id).allowed_mentions().replied_user(false)
        .build().embed(embed)?.await?;

    Ok(())
}
