use std::{
    future::Future,
    pin::Pin
};

use pad::PadStr;

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
        GetLocalUserInfractions
    },
    SystemResult
};

use crate::utilities::FutureResult;

crate struct InfractionSearchCommand;

impl Command for InfractionSearchCommand {
    fn name(&self) -> String {
        String::from("inf")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("inf search")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let query = arguments.next().unwrap_or("").to_string();

        Box::pin(infractions_infraction_search_command(ctx, query))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>,
                                                params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_infraction_search_command(ctx: CommandContext<'_>, query: String)
    -> SystemResult<()> {
    let user_id = if let Ok(uid) = UserId::parse(query.as_str()) {
        uid
    }
    else if let Ok(uid) = query.parse() {
            UserId(uid)
    }
    else {
        return Err(box CommandError("Querying infractions with not a user id is not currently supported.".to_string()));
    };

    let infractions = ctx.http_client.clone().get_local_user_infractions(
        ctx.message.guild_id.unwrap(), user_id).await?;

    if infractions.is_empty() {
        ctx.http_client.clone().create_message(ctx.message.channel_id)
            .content("The user has no infractions.")?.allowed_mentions()
            .replied_user(false).build().reply(ctx.message.id).await?;
    }
    else {
        let mut message: String =
            "```ID                                                       | Type     | Reason\n".to_owned()
                + "-------------------------------------------------------- | -------- | ----" +
                "-------------------------------------------------------------------";

        infractions.iter().for_each(|inf| {
            message.push_str(format!("\n{} | {} | {}", inf.infraction_id, inf.infraction_type.to_string()
                .as_str().pad_to_width(8), inf.reason).as_str())
        });

        message.push_str("\n```");

        ctx.http_client.clone().create_message(ctx.message.channel_id).content(message)?.allowed_mentions()
            .replied_user(false).build().reply(ctx.message.id).await?;
    }

    Ok(())
}
