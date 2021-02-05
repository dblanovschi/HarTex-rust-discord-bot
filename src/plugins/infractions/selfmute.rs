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

use crate::utilities::{
    duration::parse_duration,
    FutureResult
};

use crate::xml_deserialization::BotConfig;

crate struct SelfmuteCommand;

impl Command for SelfmuteCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("selfmute")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
                                            -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        let user = arguments.next().unwrap().to_string();
        let duration = arguments.next().unwrap_or("10s").to_string();
        let reason = arguments.into_remainder().unwrap_or("No reason specified").to_string();

        Box::pin(infractions_selfmute_command(ctx, user, duration, reason))
    }
}

async fn infractions_selfmute_command(ctx: CommandContext<'_>, user: String, duration: String, reason: String) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();

    let guild_name = if let Ok(Some(guild)) = ctx.http_client.clone().guild(guild_id).await {
        guild.name
    }
    else {
        "unknown".to_string()
    };

    let user_id = if let Ok(uid) = UserId::parse(user.as_str()) {
        uid
    }
    else if let Ok(uid) = user.parse() {
        UserId(uid)
    }
    else {
        return Err(box CommandError("The specified User ID is invalid.".to_string()));
    };

    let guild_config = ctx.http_client.clone().get_guild_configuration(guild_id).await?;
    let config = quick_xml::de::from_str::<BotConfig>(guild_config.as_str())?;

    let infraction_id = format!("{:x}", Sha3_224::digest(
        format!("{}{}{}", guild_id, user_id, reason.clone()).as_str().as_bytes()));

    let dur = parse_duration(duration.clone());

    if let Some(muted_role) = config.plugins.infractions_plugin.mute_command.muted_role {
        let role_id = RoleId(muted_role.role_id);

        ctx.http_client.clone()
            .add_user_infraction(infraction_id.clone(), guild_id, user_id, reason.clone(), InfractionType::TemporaryMute)
            .await?;

        ctx.http_client.clone().add_guild_member_role(guild_id, user_id, role_id).await?;

        if let Some(role_to_remove) = config.plugins.infractions_plugin.mute_command.role_to_remove {
            ctx.http_client.clone().remove_guild_member_role(guild_id, user_id, RoleId(role_to_remove.role_id))
                .await?;
        }

        ctx.http_client.clone().create_message(ctx.message.channel_id)
            .content(
                format!(
                    "<:green_check:705623382682632205> Successfully temporarily self-muted user {} (ID: `{}`) for `{}`. Reason: `{}`. Infraction ID: `{}`",
                    user_id.mention(), user_id.0, duration.clone(), reason, infraction_id))?
            .allowed_mentions().replied_user(false).build().reply(ctx.message.id).await?;

        let dm_channel = ctx.http_client.clone().create_private_channel(user_id).await?;

        ctx.http_client.clone()
            .create_message(dm_channel.id)
            .content(format!("You have been temporarily self-muted in guild `{}` for `{}` (ID: `{}`). Reason: `{}`",
                             guild_name, duration, guild_id.0, reason))?
            .await?;

        tokio::time::sleep(dur).await;

        ctx.http_client.clone().remove_guild_member_role(guild_id, user_id, role_id).await?;

        if let Some(role_to_remove) = config.plugins.infractions_plugin.mute_command.role_to_remove {
            ctx.http_client.clone().add_guild_member_role(guild_id, user_id, RoleId(role_to_remove.role_id))
                .await?;
        }

        Ok(())
    }
    else {
        Err(box CommandError("Muted role is not set.".to_string()))
    }
}

