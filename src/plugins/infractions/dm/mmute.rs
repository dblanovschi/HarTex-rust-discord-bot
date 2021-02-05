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

crate struct MmuteCommand;

impl Command for MmuteCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("mmute")
    }

    fn aliases(&self) -> Vec<String> {
        vec![String::from("dmmute")]
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

        Box::pin(infractions_mmute_command(ctx, users, reason.to_string()))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_mmute_command(ctx: CommandContext<'_>, users: Vec<String>, reason: String) -> SystemResult<()> {
    let mut members_to_mute = Vec::new();
    let guild_id = ctx.message.guild_id.unwrap();
    let channel_id = ctx.message.channel_id;

    let guild_name = if let Ok(Some(guild)) = ctx.http_client.clone().guild(guild_id).await {
        guild.name
    }
    else {
        "unknown".to_string()
    };

    for user in users {
        if let Ok(user_id) = UserId::parse(&user) {
            members_to_mute.push(user_id);
        }
        else if let Ok(user_id) = user.parse() {
            members_to_mute.push(UserId(user_id));
        }
        else {
            return Err(box CommandError("Specified User ID is invalid.".to_string()))
        }
    }

    for member in members_to_mute {
        let guild_config = ctx.http_client.clone().get_guild_configuration(guild_id).await?;
        let config = quick_xml::de::from_str::<BotConfig>(guild_config.as_str())?;

        let warning_id = format!("{:x}", Sha3_224::digest(
            format!("{}{}{}", guild_id, member, reason).as_str().as_bytes()));

        if let Some(muted_role) = config.plugins.infractions_plugin.mute_command.muted_role {
            let role_id = RoleId(muted_role.role_id);

            if let Ok(Some(user)) = ctx.http_client.user(member).await {
                ctx.http_client.clone().add_user_infraction(warning_id.clone(),
                                                            guild_id, member, reason.clone(),
                                                            InfractionType::Mute).await?;

                ctx.http_client.clone().add_guild_member_role(guild_id, member, role_id).await?;

                if let Some(role_to_remove) = config.plugins.infractions_plugin.mute_command
                    .role_to_remove {
                    ctx.http_client.clone().remove_guild_member_role(guild_id, member,
                                                                     RoleId(role_to_remove.role_id)).await?;
                }

                ctx.http_client.clone().create_message(ctx.message.channel_id)
                    .content(
                        format!(
                            "<:green_check:705623382682632205> Successfully muted user {} (ID: `{}`). Reason: `{}`. Infraction ID: `{}`",
                            user.mention(), member.0, reason, warning_id))?
                    .allowed_mentions().replied_user(false).build().reply(ctx.message.id).await?;

                let dm_channel = ctx.http_client.clone().create_private_channel(member).await?;

                ctx.http_client.clone()
                    .create_message(dm_channel.id)
                    .content(format!("You have been muted in guild `{}` (ID: `{}`). Reason: `{}`",
                                     guild_name, guild_id.0, reason))?
                    .await?;
            }
        }
        else {
            ctx.http_client
                .clone().create_message(channel_id)
                .content("<:red_x:705623424675872859> Muted role is not set.")?
                .allowed_mentions()
                .replied_user(false)
                .build()
                .reply(ctx.message.id)
                .await?;

            return Err(box CommandError("Muted role is not set.".to_string()))
        }
    }

    Ok(())
}
