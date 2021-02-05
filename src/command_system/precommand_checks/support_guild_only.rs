use std::{
    future::Future,
    pin::Pin
};

use twilight_model::{
    id::GuildId
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

crate struct SupportGuildOnly;

impl PrecommandCheck for SupportGuildOnly {
    fn execute_check<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _params: PrecommandCheckParameters)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(support_guild_only(ctx))
    }
}

async fn support_guild_only(ctx: CommandContext<'asynchronous_trait>) -> SystemResult<()> {
    return if ctx.message.guild_id == Some(GuildId(631100984176672768)) {
        Ok(())
    }
    else {
        Err(box CommandError("The command can only be executed in the support guild.".to_string()))
    }
}

