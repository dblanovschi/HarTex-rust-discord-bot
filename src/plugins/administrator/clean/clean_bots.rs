use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache,
};

use twilight_model::{
    id::{
        MessageId,
    },
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
    CommandError,
    PrecommandCheckParameters
};

use crate::system::{
    SystemResult,
};

use crate::utilities::FutureResult;

crate struct CleanBotsCommand;

impl Command for CleanBotsCommand {
    fn name(&self) -> String {
        String::from("clean")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("clean user")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let number = arguments.next().unwrap_or("10").to_string();

        Box::pin(administrator_clean_bots_command(ctx, number))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_clean_bots_command(ctx: CommandContext<'_>, number_of_messages: String) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let number = if let Ok(int) = number_of_messages.parse::<u64>() {
        int
    }
    else {
        ctx.http_client.clone().create_message(channel_id)
            .content(format!("<:red_x:705623424675872859> Invalid limit: `{}`", number_of_messages))?.await?;

        return Err(box CommandError("Number of messages to delete is not a number.".to_string()))
    };

    let message_ids = ctx
        .http_client
        .clone()
        .channel_messages(channel_id)
        .limit(number)?
        .await?
        .iter()
        .filter(|message| message.author.bot)
        .map(|message| message.id)
        .collect::<Vec<MessageId>>();

    ctx.http_client.clone().delete_messages(channel_id, message_ids.clone()).await?;
    ctx.http_client.clone().create_message(channel_id).content(
        format!("<:green_check:705623382682632205> Deleted `{}` messages successfully!", message_ids.len()))?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}
