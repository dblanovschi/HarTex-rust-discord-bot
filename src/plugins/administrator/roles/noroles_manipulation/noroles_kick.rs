use std::{
    future::Future,
    pin::Pin,
    sync::Arc
};

use twilight_cache_inmemory::{
    model::CachedMember,
    InMemoryCache,
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
    SystemResult,
};

use crate::utilities::FutureResult;

crate struct NorolesKickCommand;

impl Command for NorolesKickCommand {
    fn name(&self) -> String {
        String::from("noroles")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("noroles list")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
                                            -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send+ 'asynchronous_trait>> {
        Box::pin(administrator_noroles_kick_command(ctx, cache))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
                                                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_noroles_kick_command(ctx: CommandContext<'_>, cache: InMemoryCache) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();
    let members =
        cache
            .guild_members(guild_id)
            .unwrap()
            .iter()
            .map(|user_id| cache.member(guild_id, *user_id).unwrap())
            .filter(|member| (*member).roles.is_empty()).collect::<Vec<Arc<CachedMember>>>();

    for member in &members {
        ctx.http_client.clone().remove_guild_member(guild_id, member.user.id).await?;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    ctx.http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .content(format!("<:green_check:705623382682632205> Kicked `{}` members with no roles.", members.len()))?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}

