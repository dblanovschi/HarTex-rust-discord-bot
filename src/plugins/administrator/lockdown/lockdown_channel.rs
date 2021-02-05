use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache,
};

use twilight_http::{
    request::{
        channel::reaction::RequestReactionType
    }
};

use twilight_model::{
    guild::{
        Permissions
    },
    id::{
        EmojiId,
        RoleId
    },
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
    PrecommandCheckParameters
};

use crate::system::{
    twilight_id_extensions::IntoInnerU64,
    SystemResult,
};

use crate::utilities::FutureResult;

crate struct LockdownChannelCommand;

impl Command for LockdownChannelCommand {
    fn name(&self) -> String {
        String::from("lockdown")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("lockdown channel")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
                                            -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(administrator_lockdown_channel_command(ctx))
    }

    fn precommand_checks<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, checks: Box<[C]>)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        checks.iter().for_each(|check| {
            Box::pin(FutureResult::resolve(check(ctx.clone(), params.clone())));
        });

        Box::pin(FutureResult::ok())
    }
}

async fn administrator_lockdown_channel_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let allow = Permissions::VIEW_CHANNEL;
    let deny = Permissions::SEND_MESSAGES | Permissions::SEND_TTS_MESSAGES;

    ctx.http_client
        .clone()
        .update_channel_permission(channel_id, allow, deny)
        .role(RoleId(ctx.message.guild_id.unwrap().into_inner_u64()))
        .await?;
    ctx.http_client.clone().create_reaction(channel_id, ctx.message.id, RequestReactionType::Custom {
        id: EmojiId(705623382682632205),
        name: None
    });

    Ok(())
}
