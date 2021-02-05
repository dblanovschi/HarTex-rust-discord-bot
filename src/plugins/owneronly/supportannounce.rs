use std::{
    future::Future,
    pin::Pin
};

use twilight_cache_inmemory::InMemoryCache;

use twilight_embed_builder::{
    EmbedBuilder,
    EmbedFieldBuilder
};

use twilight_http::{
    request::channel::reaction::RequestReactionType
};

use twilight_model::{
    id::{
        ChannelId,
        EmojiId
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

use crate::system::SystemResult;

use crate::utilities::FutureResult;

crate struct SupportAnnounceCommand;

impl Command for SupportAnnounceCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("support-announce")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, mut arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let mut title_done = false;
        let mut title = String::new();
        let mut description = String::new();

        #[allow(clippy::while_let_on_iterator)]
        while let Some(next) = arguments.next() {
            if next == "--title" {
                title_done = false;
            }
            else if next == "--description" {
                title_done = true;
            }
            else {
                if !title_done {
                    title.push_str(next);
                    title.push(' ');
                }
                else {
                    description.push_str(next);
                    description.push(' ');
                }
            }
        }

        Box::pin(owneronly_support_announce_command(ctx, title, description))
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

async fn owneronly_support_announce_command(ctx: CommandContext<'_>, title: String, description: String) -> SystemResult<()> {
    let embed = EmbedBuilder::new()
        .title(title)?
        .description(description)?
        .color(0x03_BE_FC)?
        .build()?;

    ctx.http_client.clone().create_reaction(ctx.message.channel_id, ctx.message.id, RequestReactionType::Custom {
        id: EmojiId(705623382682632205),
        name: None
    });
    ctx.http_client.clone().create_message(ChannelId(667597366265905166))
        .content("<@&671980298489167885>")?.embed(embed)?.await?;

    Ok(())
}
