use std::{
    future::Future,
    pin::Pin,
    sync::Arc
};

use chrono::{
    DateTime,
    Local,
    TimeZone
};

use twilight_cache_inmemory::{
    InMemoryCache
};

use twilight_embed_builder::{
    EmbedBuilder, 
    EmbedFieldBuilder,
    ImageSource
};

use twilight_mention::{
    parse::{
        ParseMention
    },
    Mention
};

use twilight_model::{
    gateway::{
        presence::{
            Activity,
            ActivityType,
            Status
        }
    },
    guild::{
        Role,
        Permissions
    },
    id::{
        UserId
    }
};

use twilight_util::{
    snowflake::Snowflake
};

use crate::command_system::{
    parser::{
        Arguments
    },
    Command,
    CommandContext,
};

use crate::system::{
    SystemResult,
};

crate struct UserinfoCommand;

impl Command for UserinfoCommand {
    fn fully_qualified_name(&self) -> String {
        String::from("user-info")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>,
                                            mut arguments: Arguments<'asynchronous_trait>, cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        let id = arguments.next().unwrap_or("");
        let user_id = if let Ok(uid) = UserId::parse(<&str>::clone(&id)) {
            Some(uid)
        }
        else {
            None
        };

        Box::pin(information_userinfo_command(ctx, user_id, cache))
    }
}

async fn information_userinfo_command(ctx: CommandContext<'_>, id: Option<UserId>,
                                      cache: InMemoryCache) -> SystemResult<()> {
    let user = if let Some(uid) = id {
        match ctx.http_client.user(uid).await? {
            Some(u) => u,
            None => ctx.author.clone()
        }
    }
    else {
        ctx.author.clone()
    };

    let guild_id = ctx.message.guild_id.unwrap();
    let presence = cache.presence(guild_id, user.id);

    let member = ctx.http_client.guild_member(guild_id, user.id).await?.unwrap();
    let mut roles = member.roles.iter().map(|role_id| cache.role(*role_id)
        .unwrap()).collect::<Vec<Arc<Role>>>();

    roles.sort_by(|previous, now| {
        now.position.cmp(&previous.position)
    });

    let mut default: Permissions = Permissions::empty();
    roles.clone().iter().for_each(|role| {
        default |= role.permissions
    });

    let member_joined_at = 
        DateTime::parse_from_str(member.joined_at.unwrap().as_str(), "%Y-%m-%dT%H:%M:%S%.f%:z")?
            .format("%Y-%m-%d %H:%M:%S");
    let user_created_at = Local.timestamp_millis(user.id.timestamp()).format("%Y-%m-%d %H:%M:%S");

    let embed = EmbedBuilder::new()
        .color(0x03_BE_FC)?
        .title(format!("About {}", user.name))?
        .field(EmbedFieldBuilder::new("Username", user.clone().name)?.inline())
        .field(EmbedFieldBuilder::new("Discriminator", user.clone().discriminator)?.inline())
        .field(EmbedFieldBuilder::new("Nickname", match member.nick {
            Some(nickname) => nickname,
            None => "none".to_string()
        })?)
        .field(EmbedFieldBuilder::new("User ID", user.id.to_string())?)
        .field(EmbedFieldBuilder::new("Status", match presence.clone() {
            Some(presence) => {
                match presence.status {
                    Status::Online => "<:online:783281798687621151> online",
                    Status::Idle => "<:idle:783281612141887489> idle",
                    Status::DoNotDisturb => "<:dnd:783281406221877260> do not disturb",
                    Status::Invisible => "<:offline:783282083287007285> invisible",
                    Status::Offline => "<:offline:783282083287007285> offline"
                }
            },
            None => "unknown"
        })?)
        .field(EmbedFieldBuilder::new("Highest Role in Guild", format!("{}",
                                                                       roles.first().unwrap().mention()))?)
        .field(EmbedFieldBuilder::new("Custom Status", match presence.clone() {
            Some(presence) => {
                let activities: &Vec<Activity> = &presence.activities;

                let only_custom = activities.iter().find(|activity|
                    activity.kind == ActivityType::Custom);

                match only_custom {
                    Some(activity) => {
                        match activity.state.clone() {
                            Some(state) => state,
                            None => "none".to_string()
                        }
                    },
                    None => "none".to_string()
                }
            },
            None => {
                "none".to_string()
            }
        })?)
        .field(EmbedFieldBuilder::new("Guild Permission Integer", format!("{:?}", default))?)
        .field(EmbedFieldBuilder::new("Joined Guild At", format!("{}+08:00", member_joined_at))?)
        .field(EmbedFieldBuilder::new("Account Created At", format!("{}+08:00", user_created_at))?)
        .thumbnail(ImageSource::url(format!("https://cdn.discordapp.com/avatars/{}/{}.png",
                                            user.id, user.avatar.unwrap()))?)
        .build()?;

    ctx.http_client.create_message(ctx.message.channel_id).reply(ctx.message.id).allowed_mentions()
        .replied_user(false).build().embed(embed)?.await?;

    Ok(())
}
