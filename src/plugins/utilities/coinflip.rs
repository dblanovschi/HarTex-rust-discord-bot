use std::{
    future::Future,
    pin::Pin
};

use rand::{
    random
};

use twilight_cache_inmemory::{
    InMemoryCache
};

use twilight_http::{
    request::{
        channel::{
            allowed_mentions::{
                AllowedMentions
            }
        }
    }
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
    CommandError
};

use crate::system::{
    SystemResult,
};

crate struct CoinflipCommand;

impl Command for CoinflipCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("coinflip")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(utilities_coinflip_command(ctx))
    }
}

async fn utilities_coinflip_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let message = ctx
        .http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .content("Flipping a coin; please wait...")?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;
    let allowed_mentions = AllowedMentions::default();

    let result = random::<bool>();

    ctx.http_client
        .clone()
        .update_message(ctx.message.channel_id, message.id)
        .content(format!("The coin landed on **{}**.", if result { "head" } else { "tail" }))?
        .allowed_mentions(allowed_mentions).await?;

    Ok(())
}
