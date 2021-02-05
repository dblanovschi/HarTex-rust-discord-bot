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

crate struct NorolesListCommand;

impl Command for NorolesListCommand {
    fn name(&self) -> String {
        String::from("noroles")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("noroles list")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send+ 'asynchronous_trait>> {
        Box::pin(administrator_noroles_list_command(ctx, cache))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_noroles_list_command(ctx: CommandContext<'_>, cache: InMemoryCache) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();
    let members =
        cache
        .guild_members(guild_id)
        .unwrap()
        .iter()
        .map(|user_id| cache.member(guild_id, *user_id).unwrap())
        .filter(|member| (*member).roles.is_empty()).collect::<Vec<Arc<CachedMember>>>();
    let mut message = String::from("**__Members Without Roles__**\n");

    members.iter().for_each(|member| {
        message.push_str(&format!("{}#{}", member.user.name, member.user.discriminator));
    });

    ctx.http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .content(message)?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}
