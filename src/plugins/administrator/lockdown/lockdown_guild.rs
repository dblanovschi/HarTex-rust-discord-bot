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

crate struct LockdownGuildCommand;

impl Command for LockdownGuildCommand {
    fn name(&self) -> String {
        String::from("lockdown")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("lockdown guild")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(administrator_lockdown_guild_command(ctx, cache))
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

async fn administrator_lockdown_guild_command(ctx: CommandContext<'_>, cache: InMemoryCache) -> SystemResult<()> {
    let role = cache.role(RoleId(ctx.message.guild_id.unwrap().into_inner_u64())).unwrap();
    let mut permissions = role.permissions;
    permissions.remove(Permissions::SEND_MESSAGES);

    ctx.http_client.update_role(ctx.message.guild_id.unwrap(), role.id)
        .permissions(permissions)
        .await?;
    ctx.http_client.clone().create_reaction(ctx.message.channel_id, ctx.message.id, RequestReactionType::Custom {
        id: EmojiId(705623382682632205),
        name: None
    });

    Ok(())
}
