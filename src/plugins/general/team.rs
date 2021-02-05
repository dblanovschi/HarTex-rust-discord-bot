use std::{
    future::Future,
    pin::Pin
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

crate struct TeamCommand;

impl Command for TeamCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("team")
    }

    fn aliases(&self) -> Vec<String> {
        vec![String::from("staff")]
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments,
                                            _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_team_command(ctx))
    }
}

async fn general_team_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let embed = EmbedBuilder::new()
        .title("HarTex Team")?
        .color(0x03_BE_FC)?
        .field(EmbedFieldBuilder::new("Global Administrator & Lead Developer", "HTGAzureX1212.")?)
        .field(EmbedFieldBuilder::new("Partner Bot Developer", "Mrcomputer1")?)
        .build()?;

    ctx.http_client.create_message(ctx.message.channel_id).reply(ctx.message.id).embed(embed)?.allowed_mentions()
        .replied_user(false).build().await?;

    Ok(())
}
