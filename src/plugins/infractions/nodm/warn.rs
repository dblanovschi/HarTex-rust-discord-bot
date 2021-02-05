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
    Mention,
    parse::ParseMention,
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
        AddUserInfraction
    },
    SystemResult
};

use crate::utilities::FutureResult;

crate struct WarnCommand;

impl Command for WarnCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("nodmwarn")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let id = arguments.next().unwrap_or("");
        let user_id = if let Ok(uid) = UserId::parse(id) {
            Some(uid)
        }
        else if let Ok(uid) = id.parse() {
            Some(UserId(uid))
        }
        else {
            None
        };

        let remainder = arguments.into_remainder().unwrap_or("No reason specified").to_string();

        Box::pin(infractions_warn_command(ctx, user_id, remainder))
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

async fn infractions_warn_command(ctx: CommandContext<'_>, id: Option<UserId>, reason: String)
                                  -> SystemResult<()> {
    if let Some(uid) = id {
        let channel_id = ctx.message.channel_id;
        let guild_id = ctx.message.guild_id.unwrap();

        let warning_id = format!("{:x}", Sha3_224::digest(
            format!("{}{}{}", guild_id.0, uid.0, reason.clone()).as_bytes()));

        return if ctx.author.id != uid {
            if let Ok(Some(user)) = ctx.http_client.user(uid).await {
                ctx.http_client.clone().add_user_infraction(
                    warning_id.clone(), ctx.message.guild_id.unwrap(), uid,
                    reason.clone(), InfractionType::Warning).await?;

                ctx.http_client.clone().create_message(channel_id).content(
                    format!(
                        "<:green_check:705623382682632205> Successfully warned user {} (ID: `{}`). Reason: `{}`. Infraction ID: `{}`"
                        , user.mention(), uid.0, reason.clone(), warning_id.clone()))?
                    .reply(ctx.message.id).allowed_mentions().replied_user(false).build().await?;
            }

            Ok(())
        }
        else {
            Err(box CommandError("Cannot give a warning to the command executor himself/herself.".to_string()))
        }
    }
    else {
        Err(box CommandError("There is no user to give a warning to.".to_string()))
    }
}
