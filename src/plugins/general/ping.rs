use std::{
    future::Future,
    time::Duration,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache
};

use twilight_http::{
    request::{
        channel::{
            allowed_mentions::{
                AllowedMentions
            }
        }
    }
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
};

use crate::system::SystemResult;

crate struct PingCommand;

impl Command for PingCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("ping")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments,
                                            _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(general_ping_command(ctx))
    }
}

async fn general_ping_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;
    let message = ctx.http_client.create_message(channel_id).reply(ctx.message.id)
        .allowed_mentions().replied_user(false).build().content("Hello! Did you need anything?")?.await?;

    let info = ctx.cluster.shards().first().unwrap().info().unwrap();
    let latency = info.latency().recent().front();
    let allowed_mentions = AllowedMentions::default();
    
    let mut new_content = message.content;

    new_content.push_str(format!(" - `{:?}ms`", match latency {
        Some(ms) => ms.as_millis(),
        None =>  {
           let duration = Duration::new(0, 208000000);

           duration.as_millis()
        }
    }).as_str());

    ctx.http_client.update_message(channel_id, message.id).content(new_content)?
        .allowed_mentions(allowed_mentions).await?;

    Ok(())
}
