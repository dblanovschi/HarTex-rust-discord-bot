use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache,
};

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
    SystemResult,
};

use crate::utilities::FutureResult;

crate struct NicknameChangeCommand;

impl Command for NicknameChangeCommand {
    fn name(&self) -> String {
        String::from("nickname")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("nickname change")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let user = arguments.next().unwrap_or("unknown");
        let new_nickname = arguments.into_remainder().unwrap_or("");

        Box::pin(administrator_nickname_change_command(ctx, user.to_string(), new_nickname.to_string()))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_nickname_change_command(ctx: CommandContext<'_>, user: String, new_nickname: String) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let user_id = if let Ok(id) = UserId::parse(&user) {
        id
    }
    else if let Ok(int) = user.parse() {
        UserId(int)
    }
    else {
        ctx.http_client.clone()
            .create_message(channel_id)
            .content(format!("<:red_x:705623424675872859> Invalid user ID encountered: `{}`.", user))?
            .await?;

        return Err(box CommandError("Invalid user ID.".to_string()))
    };

    let user = if let Some(u) = ctx.http_client.clone().user(user_id).await? {
        u
    }
    else {
        ctx.http_client.clone()
            .create_message(channel_id)
            .content(format!("<:red_x:705623424675872859> User `{}` not found.", user))?
            .await?;

        return Err(box CommandError("User not found.".to_string()))
    };

    ctx.http_client
        .clone()
        .update_guild_member(ctx.message.guild_id.unwrap(), user_id)
        .nick(new_nickname.clone())?
        .await?;
    ctx.http_client.clone()
        .create_message(channel_id)
        .content(
            format!(
                "<:red_x:705623424675872859> Successfully changed nickname of user `{}#{}` to `{}`.",
                user.name,
                user.discriminator,
                new_nickname
            )
        )?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}
