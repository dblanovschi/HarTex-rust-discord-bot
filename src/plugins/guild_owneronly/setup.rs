use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::InMemoryCache;

use twilight_model::{
    id::{
        GuildId,
    }
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
    twilight_http_client_extensions::{
        InitializeWhitelistedGuild
    },
    SystemResult
};

use crate::utilities::{
    FutureResult
};

crate struct SetupCommand;

impl Command for SetupCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("setup")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let guild_id = ctx.message.guild_id.unwrap();

        Box::pin(guild_owneronly_setup_command(ctx, guild_id))
    }

    fn precommand_checks<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>,
                                                 params: PrecommandCheckParameters, checks: Box<[C]>)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> where
        C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
            -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        checks.iter().for_each(|check| {
            Box::pin(FutureResult::resolve(check(ctx.clone(), params.clone())));
        });

        Box::pin(FutureResult::ok())
    }
}

async fn guild_owneronly_setup_command(ctx: CommandContext<'_>, guild_id: GuildId)
    -> SystemResult<()> {
    ctx.http_client.clone().initialize_whitelisted_guild(guild_id).await?;

    Ok(())
}
