use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache,
};

use twilight_http::{
    request::{
        channel::reaction::RequestReactionType
    }
};

use twilight_model::{
    guild::{
        Permissions
    },
    id::{
        EmojiId,
        RoleId
    },
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
    SystemResult,
};

use crate::utilities::FutureResult;

crate struct VoicemuteEnableCommand;

impl Command for VoicemuteEnableCommand {
    fn name(&self) -> String {
        String::from("voicemute")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("voicmute enable")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(administrator_voicemute_enable_command(ctx, cache))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn administrator_voicemute_enable_command(ctx: CommandContext<'_>, cache: InMemoryCache) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();
    let channel_id = ctx.message.channel_id;
    let voice_state = cache.voice_state(ctx.author.id, guild_id).unwrap();
    let allow = Permissions::CONNECT;
    let deny = Permissions::USE_VAD;

    let voice_channel = if let Some(ch) = voice_state.channel_id {
        ch
    }
    else {
        return Err(box CommandError("User not in voice channel".to_string()))
    };

    ctx.http_client
        .clone()
        .update_channel_permission(voice_channel, allow, deny)
        .role(RoleId(guild_id.0))
        .await?;
    ctx.http_client.clone().create_reaction(channel_id, ctx.message.id, RequestReactionType::Custom {
        id: EmojiId(705623382682632205),
        name: None
    });

    Ok(())
}
