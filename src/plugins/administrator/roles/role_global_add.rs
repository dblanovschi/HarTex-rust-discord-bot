use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache,
};

use twilight_mention::{
    Mention
};

use twilight_model::{
    id::{
        RoleId,
    }
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

crate struct RoleGlobalAddCommand;

impl Command for RoleGlobalAddCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("role global-add")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            mut arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let role_id = arguments.next().unwrap_or("");

        Box::pin(administrator_role_global_add_command(ctx, role_id.to_string(), cache))
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

async fn administrator_role_global_add_command(ctx: CommandContext<'_>, role_id: String, cache: InMemoryCache)
    -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let guild_id = ctx.message.guild_id.unwrap();
    let guild_members_ids = cache.guild_members(guild_id).unwrap();

    if let Ok(rid) = role_id.parse() {
        let role_id = RoleId(rid);
        let role = if let Some(role) = cache.role(role_id) {
            role
        }
        else {
            ctx.http_client.clone().create_message(channel_id)
                .content(format!("<:red_x:705623424675872859> Role `{}` not found.", role_id))?
                .allowed_mentions().replied_user(false).build()
                .reply(ctx.message.id)
                .await?;

            return Err(box CommandError("Role not found.".to_string()));
        };

        for user_id in guild_members_ids {
            ctx.http_client.clone().add_guild_member_role(guild_id, user_id, role_id).await?;
        };

        ctx.http_client.clone().create_message(channel_id)
            .content(format!("<:green_check:705623382682632205> Added role `{}` to every member in the guild..", role.mention()))?
            .allowed_mentions().replied_user(false).build()
            .reply(ctx.message.id)
            .await?;

        Ok(())
    }
    else {
        ctx.http_client.clone().create_message(channel_id)
            .content(format!("<:red_x:705623424675872859> Cannot find role `{}` in guild.", role_id))?
            .allowed_mentions().replied_user(false).build()
            .reply(ctx.message.id)
            .await?;

        Err(box CommandError("Specified role id is invalid.".to_string()))
    }
}
