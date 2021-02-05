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

crate struct BotOwnerOnly;

impl PrecommandCheck for BotOwnerOnly {
    fn execute_check<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                          _params: PrecommandCheckParameters)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(bot_owner_only(ctx))
    }
}

async fn bot_owner_only(ctx: CommandContext<'asynchronous_trait>) -> SystemResult<()> {
    return if ctx.author.id.0 == 408576714243833867 {
        Ok(())
    }
    else {
        Err(box CommandError("Not bot owner.".to_string()))
    }
}

