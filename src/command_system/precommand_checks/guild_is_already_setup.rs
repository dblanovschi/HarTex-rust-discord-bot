use std::{
    future::Future,
    pin::Pin
};

use twilight_model::{
    id::{
        GuildId
    }
};

use crate::{
    command_system::{
        CommandContext,
        CommandError,
        PrecommandCheckParameters
    },
    system::{
        twilight_http_client_extensions::{
            GetGuildConfiguration
        },
        SystemResult
    }
};

use super::PrecommandCheck;

crate struct GuildIsAlreadySetup;

impl PrecommandCheck for GuildIsAlreadySetup {
    fn execute_check<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters)
        -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(guild_is_already_setup(ctx, params.guild_id.unwrap()))
    }
}

async fn guild_is_already_setup(ctx: CommandContext<'asynchronous_trait>, guild_id: GuildId) -> SystemResult<()> {
    return if ctx.http_client.clone().get_guild_configuration(guild_id).await.is_ok() {
        Ok(())
    }
    else {
        Err(box CommandError("Guild is not yet setup.".to_string()))
    }
}
