use std::{
    future::Future,
    pin::Pin
};

use sha3::{
    Digest,
    Sha3_224
};

use twilight_cache_inmemory::InMemoryCache;

use twilight_mention::{
    parse::ParseMention,
    Mention
};

use twilight_model::{
    id::{
        UserId
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
    model::{
        infractions::InfractionType
    },
    twilight_http_client_extensions::{
        AddUserInfraction,
    },
    SystemResult
};

use crate::utilities::FutureResult;

crate struct MunbanCommand;

impl Command for MunbanCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("nodmmunban")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
                                            -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let mut users = Vec::<String>::new();

        while let Some(string) = arguments.next() {
            match string {
                "-r" | "--reason" => break,
                _ => users.push(string.to_string())
            }
        }

        let reason = arguments.into_remainder().unwrap_or("No reason specified.");

        Box::pin(infractions_munban_command(ctx, users, reason.to_string()))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
                                                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_munban_command(ctx: CommandContext<'_>, users: Vec<String>, reason: String) -> SystemResult<()> {
    let mut users_to_ban = Vec::new();
    let guild_id = ctx.message.guild_id.unwrap();
    let channel_id = ctx.message.channel_id;

    for user in users {
        if let Ok(user_id) = UserId::parse(&user) {
            users_to_ban.push(user_id);
        }
        else if let Ok(user_id) = user.parse() {
            users_to_ban.push(UserId(user_id));
        }
        else {
            return Err(box CommandError("Specified User ID is invalid.".to_string()))
        }
    };

    for user in users_to_ban {
        let infraction_id = format!("{:x}", Sha3_224::digest(
            format!("{}{}{}", guild_id.0, user.0, reason.clone()).as_bytes()));

        if ctx.author.id != user {
            if let Ok(Some(user_)) = ctx.http_client.user(user).await {
                ctx.http_client.clone().add_user_infraction(
                    infraction_id.clone(), ctx.message.guild_id.unwrap(), user,
                    reason.clone(), InfractionType::Ban).await?;

                ctx.http_client.clone().create_message(channel_id).content(
                    format!(
                        "<:green_check:705623382682632205> Successfully banned user {} (ID: `{}`). Reason: `{}`. Infraction ID: `{}`"
                        , user_.mention(), user.0, reason.clone(), infraction_id.clone()))?
                    .reply(ctx.message.id).allowed_mentions().replied_user(false).build().await?;

                ctx.http_client.clone().delete_ban(guild_id, user).await?;
            }
        }
        else {
            return Err(box CommandError("Cannot give a warning to the command executor himself/herself.".to_string()))
        }
    };

    Ok(())
}
