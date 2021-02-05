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
    SystemResult,
};

use crate::utilities::{
    duration::parse_duration,
    FutureResult
};

crate struct SlowmodeEnableHereCommand;

impl Command for SlowmodeEnableHereCommand {
    fn name(&self) -> String {
        String::from("slowmode")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("slowmode enable here")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let duration = arguments.into_remainder().unwrap_or("10s").to_string();

        Box::pin(administrator_slowmode_enable_here_command(ctx, duration))
    }

    fn precommand_checks<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, checks: Box<[C]>)
                                                 -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        checks.iter().for_each(|check| {
            Box::pin(FutureResult::resolve(check(ctx.clone(), params.clone())));
        });

        Box::pin(FutureResult::ok())
    }
}

async fn administrator_slowmode_enable_here_command(ctx: CommandContext<'_>, duration: String) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;

    let duration = parse_duration(duration);
    let seconds = duration.as_secs();

    ctx.http_client.clone().update_channel(channel_id).rate_limit_per_user(seconds)?.await?;
    ctx.http_client.clone().create_message(channel_id)
        .content(format!("<:green_check:705623382682632205> Updated channel slowmode to `{}` seconds.", seconds))?
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .await?;

    Ok(())
}
