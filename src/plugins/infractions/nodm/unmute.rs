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
        RoleId,
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
        GetGuildConfiguration
    },
    SystemResult
};

use crate::utilities::FutureResult;

use crate::xml_deserialization::BotConfig;

crate struct UnmuteCommand;

impl Command for UnmuteCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("nodmunmute")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let to_parse = arguments.next().unwrap();
        let user_id = if let Ok(uid) = UserId::parse(to_parse) {
            Some(uid)
        }
        else if let Ok(uid) = to_parse.parse() {
            Some(UserId(uid))
        }
        else {
            None
        };

        let reason = arguments.into_remainder().unwrap().to_string();

        Box::pin(infractions_unmute_command(ctx, user_id, reason))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>,
                                                params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters) ->
            Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_unmute_command(ctx: CommandContext<'_>, user_id: Option<UserId>,
                                    reason: String)
    -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();

    if let Some(uid) = user_id {
        let guild_config = ctx.http_client.clone().get_guild_configuration(guild_id).await?;
        let config = quick_xml::de::from_str::<BotConfig>(guild_config.as_str())?;

        let warning_id = format!("{:x}", Sha3_224::digest(
            format!("{}{}{}", guild_id, uid, reason).as_str().as_bytes()));

        if let Some(muted_role) = config.plugins.infractions_plugin.mute_command
            .muted_role {
            let role_id = RoleId(muted_role.role_id);

            if let Ok(Some(user)) = ctx.http_client.user(uid).await {
                ctx.http_client.clone().remove_guild_member_role(guild_id, uid, role_id).await?;

                ctx.http_client.clone().add_user_infraction(
                    warning_id.clone(), guild_id, uid, reason.clone(),
                    InfractionType::Unmute).await?;

                if let Some(role_to_remove) = config.plugins.infractions_plugin.mute_command
                    .role_to_remove {
                    ctx.http_client.clone().add_guild_member_role(guild_id, uid,
                                                                  RoleId(role_to_remove.role_id)).await?;
                }

                ctx.http_client.clone().create_message(ctx.message.channel_id).content(
                    format!(
                        "<:green_check:705623382682632205> Successfully unmuted user {} (ID: `{}`). Reason: `{}`. Infraction ID: `{}`",
                        user.mention(), uid.0, reason, warning_id))?
                    .allowed_mentions().replied_user(false).build().reply(ctx.message.id).await?;
            }

            Ok(())
        } else {
            Err(box CommandError("Muted role is not set.".to_string()))
        }
    }
    else {
        Err(box CommandError("User ID cannot be none.".to_string()))
    }
}
