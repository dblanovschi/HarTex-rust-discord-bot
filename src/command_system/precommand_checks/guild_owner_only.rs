use std::{
    future::Future,
    pin::Pin
};

use crate::{
    command_system::{
        CommandContext,
        CommandError,
        PrecommandCheckParameters
    },
    system::{
        SystemResult
    }
};

use super::PrecommandCheck;

crate struct GuildOwnerOnly;

impl PrecommandCheck for GuildOwnerOnly {
    fn execute_check<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(guild_owner_only(ctx, params))
    }
}

async fn guild_owner_only(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters)
    -> SystemResult<()> {
    if let (Some(guild_id), Some(cache)) = (ctx.message.guild_id, params.cache) {
        let guild = cache.guild(guild_id).unwrap();

        if ctx.author.id == guild.owner_id {
            Ok(())
        }
        else {
            Err(box CommandError("Command executor is not the guild owner.".to_string()))
        }
    }
    else {
        Err(box CommandError("Both the Guild ID and in-memory cache cannot be None.".to_string()))
    }
}
