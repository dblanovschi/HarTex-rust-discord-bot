use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
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
    twilight_http_client_extensions::GetGuildConfiguration,
    SystemResult,
};

use crate::utilities::FutureResult;

use crate::xml_deserialization::BotConfig;

crate struct WebconfigListCommand;

impl Command for WebconfigListCommand {
    fn name(&self) -> String {
        String::from("webconfig")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("webconfig list")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(administrator_webconfig_list_command(ctx))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_webconfig_list_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let configuration =
        quick_xml::de::from_str::<BotConfig>(
            ctx.http_client
                .clone()
                .get_guild_configuration(ctx.message.guild_id.unwrap())
                .await?
                .as_str()
        )?;
    let mut content = String::from("**__Members Who Have Access to the Web Configuration of This Guild__**\n");

    for user in configuration.dashboard.users {
        content.push_str(
            &format!("User: `{}`; Permission Level: `{}`\n",
                     user.id,
                     user.permission_level
            )
        );
    }

    ctx.http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .content(content)?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}
