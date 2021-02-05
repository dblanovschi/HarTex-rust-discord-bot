use std::{
    future::Future,
    pin::Pin
};

use twilight_model::channel::{
    Channel,
    GuildChannel
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

crate struct GuildTextChannelOnly;

impl PrecommandCheck for GuildTextChannelOnly {
    fn execute_check<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _params: PrecommandCheckParameters)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(guild_text_channel_only(ctx))
    }
}

async fn guild_text_channel_only(ctx: CommandContext<'_>) -> SystemResult<()> {
    let channel = ctx.http_client.channel(ctx.message.channel_id).await?.unwrap();

    return match channel {
        Channel::Guild(guild_channel) => {
            match guild_channel {
                GuildChannel::Text(_) => Ok(()),
                _ => Err(box CommandError("Not a guild text channel.".to_string()))
            }
        },
        _ => Err(box CommandError("Not a guild channel".to_string()))
    }
}
