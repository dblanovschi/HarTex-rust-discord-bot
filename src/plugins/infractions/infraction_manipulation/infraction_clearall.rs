use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::InMemoryCache;

use twilight_mention::{
    ParseMention
};

use twilight_model::{
    id::UserId
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
    twilight_http_client_extensions::{
        ClearUserInfractions
    },
    SystemResult
};

use crate::utilities::FutureResult;

crate struct InfractionClearallCommand;

impl Command for InfractionClearallCommand {
    fn name(&self) -> String {
        String::from("inf")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("inf clear-all")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let string = arguments.next().unwrap_or("unknown");

        Box::pin(infractions_infraction_clearall_command(ctx, string.to_string()))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_infraction_clearall_command(ctx: CommandContext<'_>, user: String) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();
    let user_id = if let Ok(id) = UserId::parse(&user) {
        id
    }
    else if let Ok(id) = user.parse() {
        UserId(id)
    }
    else {
        return Err(box CommandError("Specified User ID is invalid.".to_string()))
    };

    ctx.http_client.clone().clear_user_infractions(guild_id, user_id).await?;
    ctx.http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .content("<:green_check:705623382682632205> Operation successful.")?
        .await?;

    Ok(())
}
