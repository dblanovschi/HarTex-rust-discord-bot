use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::{
    InMemoryCache
};

use twilight_model::{
    id::EmojiId
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
    CommandError
};

use crate::content_distribution_network::ContentDistributionNetwork;

use crate::parsers::{
    EmojiParser,
    Parser
};

use crate::system::{
    SystemResult,
};

crate struct EmojiCommand;

impl Command for EmojiCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("emoji")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let emoji = arguments.next().unwrap_or("unknown").to_string();

        Box::pin(utilities_emoji_command(ctx, emoji))
    }
}

async fn utilities_emoji_command(ctx: CommandContext<'_>, emoji_id: String) -> SystemResult<()> {
    let channel_id = ctx.message.channel_id;

    let emoji = match EmojiParser::new().parse(emoji_id.clone()) {
        Ok(emoji) => emoji,
        Err(error) => {
            ctx.http_client
                .clone()
                .create_message(channel_id)
                .allowed_mentions()
                .replied_user(false)
                .build()
                .reply(ctx.message.id)
                .content(format!("Couldn't parse the emoji due to an error: {}", error))?
                .await?;

            return Err(box error)
        }
    };

    let mut message = String::new();

    message.push_str(&format!("**Emoji ID:** {}\n", emoji.id));
    message.push_str(&format!("**Emoji Name:** {}\n", emoji.name));
    message.push_str(&format!("**Emoji Is Animated:** {}\n", emoji.animated));
    message.push_str(&format!("**Emoji URI:** ||{}||", ContentDistributionNetwork::custom_emoji(EmojiId(emoji.id), emoji.animated)?));

    ctx.http_client
        .clone()
        .create_message(channel_id)
        .allowed_mentions()
        .replied_user(false)
        .build()
        .reply(ctx.message.id)
        .content(message)?
        .await?;

    Ok(())
}
