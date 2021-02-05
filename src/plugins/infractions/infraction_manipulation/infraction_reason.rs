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
    model::infraction_update_type::InfractionUpdateType,
    twilight_http_client_extensions::{
        UpdateUserInfraction
    },
    SystemResult
};

use crate::utilities::FutureResult;

crate struct InfractionReasonCommand;

impl Command for InfractionReasonCommand {
    fn name(&self) -> String {
        String::from("inf")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("inf reason")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        unimplemented!()
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_infraction_reason(ctx: CommandContext<'_>, user_id: String, infraction_id: String, new_reason: String) -> SystemResult<()> {
    let user_id = if let Ok(id) = UserId::parse(&user_id) {
        id
    }
    else if let Ok(id) = user_id.parse() {
        UserId(id)
    }
    else {
        return Err(box CommandError("Specified User ID is invalid.".to_string()))
    };

    ctx.http_client
        .clone()
        .update_user_infraction(infraction_id, ctx.message.guild_id.unwrap(), user_id, InfractionUpdateType::Reason { new_reason })
        .await?;
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
