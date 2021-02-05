use std::{
    future::Future,
    pin::Pin
};

use rand::{
    Rng,
    thread_rng
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

crate struct RandintCommand;

impl Command for RandintCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("randint")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let start = arguments.next().unwrap_or("1").parse().unwrap_or(1);
        let end = arguments.next().unwrap_or("10").parse().unwrap_or(10);

        Box::pin(utilities_randint_command(ctx, start, end))
    }
}

async fn utilities_randint_command(ctx: CommandContext<'_>, range_start: u128, range_end: u128) -> SystemResult<()> {
    let message = ctx
        .http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .content("Generating random number; please wait...")?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;
    let allowed_mentions = AllowedMentions::default();

    let generated_number = thread_rng().gen_range(range_start..=range_end);

    ctx.http_client
        .clone()
        .update_message(ctx.message.channel_id, message.id)
        .content(format!("The random number from {} to {} is: `{}`", range_start, range_end, generated_number))?
        .allowed_mentions(allowed_mentions).await?;

    Ok(())
}
