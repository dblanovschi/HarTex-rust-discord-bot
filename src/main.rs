#![feature(arbitrary_self_types)]
#![feature(associated_type_defaults)]
#![feature(box_syntax)]
#![feature(crate_visibility_modifier)]
#![feature(const_fn)]
#![feature(exclusive_range_pattern)]
#![feature(in_band_lifetimes)]
#![feature(let_chains)]

#![allow(clippy::needless_lifetimes)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_wraps)]
#![allow(incomplete_features)]

extern crate ctrlc;
#[macro_use]
extern crate serde_derive;
extern crate sha3;

use std::{
    env::*,
    error::Error,
    pin::Pin,
    sync::Arc
};

use dotenv::dotenv;

use futures_util::{
    future::Either,
    stream::StreamExt
};

use sha3::{
    Digest,
    Sha3_512
};

use twilight_cache_inmemory::{
    ResourceType,
    InMemoryCache
};

use twilight_gateway::{
    cluster::{
        Cluster, 
        ShardScheme
    },
    Event,
    EventTypeFlags,
    Intents
};

use twilight_http::{
    request::{
        channel::reaction::RequestReactionType
    },
    Client as TwilightHttpClient
};

use twilight_model::{
    channel::{
        Message
    },
    gateway::{
        payload::{
            update_status::UpdateStatusInfo
        },
        presence::{
            Status
        },
    },
    id::{
        EmojiId
    }
};

crate mod command_system;
crate mod content_distribution_network;
crate mod logging;
crate mod parsers;
crate mod plugins;
crate mod system;
crate mod utilities;
crate mod xml_deserialization;

use crate::command_system::{
    events::{
        emitter::CommandEventEmitter,
        events::SystemEvent
    },
    parser::{
        Command,
        CommandParser
    },
    precommand_checks::{
        BotOwnerOnly,
        GuildIsAlreadySetup,
        GuildTextChannelOnly,
        GuildOwnerOnly,
        HasRolePermissions,
        PrecommandCheck,
        SupportGuildOnly
    },
    Command as CommandTrait,
    CommandContext,
    CommandContextRef,
    CommandFramework,
    PrecommandCheckParameters,
    PrecommandCheckParametersBuilder
};

use crate::system::{
    bot_configuration::BotConfiguration,
    event_handler::EventHandler,
    internal_bot_error::report_ibe,
    model::{
        payload::{
            CommandExecuted,
            CommandFailed,
            CommandReceived
        }
    },
    EventType,
    Stopwatch,
    SystemError,
    set_bot_activity,
};

use crate::logging::{
    logger::Logger
};

use crate::plugins::{
    administrator::{
        clean::{
            CleanAllCommand,
            CleanBotsCommand,
            CleanUserCommand
        },
        invites::{
            InvitesCommand
        },
        nickname_manipulation::{
            NicknameChangeCommand,
            NicknameRemoveCommand
        },
        lockdown::{
            LockdownChannelCommand,
            LockdownGuildCommand,
            UnlockdownChannelCommand,
            UnlockdownGuildCommand
        },
        roles::{
            noroles_manipulation::{
                NorolesKickCommand,
                NorolesListCommand
            },
            RoleAddCommand,
            RoleGlobalAddCommand,
            RoleGlobalRemoveCommand,
            RoleinfoCommand,
            RoleRemoveCommand
        },
        slowmode::{
            SlowmodeDisableChannelCommand,
            SlowmodeDisableHereCommand,
            SlowmodeEnableChannelCommand,
            SlowmodeEnableHereCommand
        },
        voice_manipulation::{
            VoicemuteDisableCommand,
            VoicemuteEnableCommand
        },
        WebconfigListCommand
    },
    general::{
        AboutCommand,
        HelpCommand,
        PingCommand,
        TeamCommand,
        UptimeCommand
    },
    guild_owneronly::{
        SetupCommand
    },
    information::{
        BotinfoCommand,
        GuildinfoCommand,
        UserinfoCommand
    },
    infractions::{
        dm::{
            DmBanCommand,
            DmCleanBanCommand,
            DmKickCommand,
            DmMbanCommand,
            DmMkickCommand,
            DmMmuteCommand,
            DmMunbanCommand,
            DmMunmuteCommand,
            DmMuteCommand,
            DmMwarnCommand,
            DmTempbanCommand,
            DmTempmuteCommand,
            DmUnbanCommand,
            DmUnmuteCommand,
            DmWarnCommand,
        },
        infraction_manipulation::{
            InfractionClearallCommand,
            InfractionReasonCommand,
            InfractionRemoveCommand,
            InfractionSearchCommand,
            InfractionsArchiveCommand
        },
        nodm::{
            NodmBanCommand,
            NodmCleanBanCommand,
            NodmKickCommand,
            NodmMbanCommand,
            NodmMkickCommand,
            NodmMmuteCommand,
            NodmMunbanCommand,
            NodmMunmuteCommand,
            NodmMuteCommand,
            NodmMwarnCommand,
            NodmTempbanCommand,
            NodmTempmuteCommand,
            NodmUnbanCommand,
            NodmUnmuteCommand,
            NodmWarnCommand
        },
        SelfmuteCommand,
    },
    owneronly::{
        RefreshWhitelistRolesCommand,
        RestartCommand,
        StopCommand,
        SupportAnnounceCommand,
        SupportinfoCommand
    },
    utilities::{
        CoinflipCommand,
        EmojiCommand,
        RandintCommand
    },
    whitelist::{
        AcceptCommand
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Creates a new stopwatch.
    let stopwatch = Stopwatch::new();

    Logger::log_debug("Loading bot token from environment.");
    dotenv().ok();  // Loads the .env file.

    // Sets the token only if no errors are encountered when attempting to retrieve the token.
    let hartex_token = match var("HARTEX_TOKEN") {
        Ok(token) => {
            token
        },
        Err(_) => {
            Logger::log_error("Failed to retrieve bot token.");
            panic!("Failed to retrieve bot token.");
        }
    };

    Logger::log_debug("Bot token successfully retrieved.".to_string());

    // Creates a new configuration object.
    let bot_configuration = BotConfiguration::new(hartex_token);

    // Sharding scheme
    let shard_scheme = ShardScheme::Auto;

    Logger::log_debug("Building bot cluster.");
    Logger::log_debug("Registering gateway intents [ALL].");
    Logger::log_debug("Registering presence.");

    let intents =
        Intents::GUILDS |
        Intents::GUILD_MEMBERS |
        Intents::GUILD_BANS |
        Intents::GUILD_EMOJIS |
        Intents::GUILD_INTEGRATIONS |
        Intents::GUILD_WEBHOOKS |
        Intents::GUILD_INVITES |
        Intents::GUILD_VOICE_STATES |
        Intents::GUILD_PRESENCES |
        Intents::GUILD_MESSAGES |
        Intents::GUILD_MESSAGE_REACTIONS |
        Intents::GUILD_MESSAGE_TYPING |
        Intents::DIRECT_MESSAGES |
        Intents::DIRECT_MESSAGE_REACTIONS |
        Intents::DIRECT_MESSAGE_TYPING;

    let activities = vec![set_bot_activity()];

    // HarTex Bot cluster for Discord gateway
    let hartex_cluster = Cluster::builder(&bot_configuration.token, intents)
        .shard_scheme(shard_scheme)
        .presence(UpdateStatusInfo::new(activities, false, None, Status::Online))
        .build()
        .await?;

    Logger::log_debug("Bot cluster successfully created.");

    // Cloned cluster for tokio spawn
    let cluster_spawn = hartex_cluster.clone();

    // Spawns a tokio task to startup the cluster.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    Logger::log_debug("Building HTTP client.");

    // HarTex HTTP client
    let hartex_http = TwilightHttpClient::new(&bot_configuration.token);

    // HarTex command framework
    Logger::log_debug("Initializing command framework.");
    let framework = CommandFramework::new();

    // Initialization of command parser.
    let command_parser = {
        Logger::log_debug("Registering commands.");

        framework
            .clone()
            .command_prefix("hb.")

            // Administrator Command Module
            .command(CleanAllCommand, true, false, false)
            .command(RoleAddCommand, true, false, false)
            .command(RoleRemoveCommand, true, false, false)
            .command(RoleGlobalAddCommand, true, false, false)
            .command(RoleGlobalRemoveCommand, true, false, false)
            .command(RoleinfoCommand, true, true, false)
            .command(CleanUserCommand, true, false, false)
            .command(CleanBotsCommand, true, false, false)
            .command(LockdownChannelCommand, true, false, false)
            .command(UnlockdownChannelCommand, true, false, false)
            .command(SlowmodeEnableHereCommand, true, false, false)
            .command(SlowmodeDisableHereCommand, true, false, false)
            .command(SlowmodeEnableChannelCommand, true, false, false)
            .command(SlowmodeDisableChannelCommand, true, false, false)
            .command(VoicemuteEnableCommand, true, false, false)
            .command(VoicemuteDisableCommand, true, false, false)
            .command(NorolesListCommand, true, false, false)
            .command(NorolesKickCommand, true, false, false)
            .command(NicknameChangeCommand, true, false, false)
            .command(NicknameRemoveCommand, true, false, false)
            .command(WebconfigListCommand, true, false, false)
            .command(InvitesCommand, true, true, false)
            .command(LockdownGuildCommand, true, false, false)
            .command(UnlockdownGuildCommand, true, false, false)

            // General Command Module
            .command(PingCommand, true, true, false)
            .command(HelpCommand, true, true, false)
            .command(AboutCommand, true, true, false)
            .command(TeamCommand, true, true, true)
            .command(UptimeCommand, true, true, false)

            // Guild Owneronly Command Module
            .command(SetupCommand, true, true, false)

            // Information Command Module
            .command(UserinfoCommand, true, true, false)
            .command(GuildinfoCommand, true, true, true)
            .command(BotinfoCommand, true, true, false)

            // Infractions Command Module
            .command(InfractionSearchCommand, true, false, false)
            .command(InfractionRemoveCommand, true, false, false)
            .command(InfractionsArchiveCommand, true, false, false)
            .command(InfractionClearallCommand, true, false, false)
            .command(InfractionReasonCommand, true, false, false)

            .command(DmWarnCommand, true, true, true)
            .command(DmMuteCommand, true, true, true)
            .command(DmUnmuteCommand, true, true, true)
            .command(DmBanCommand, true, true, true)
            .command(DmKickCommand, true, true, true)
            .command(DmMkickCommand, true, true, true)
            .command(DmCleanBanCommand, true, true, true)
            .command(DmUnbanCommand, true, true, true)
            .command(DmTempmuteCommand, true, true, true)
            .command(DmMmuteCommand, true, true, true)
            .command(DmMwarnCommand, true, true, true)
            .command(DmMbanCommand, true, true, true)
            .command(DmTempbanCommand, true, true, true)
            .command(DmMunbanCommand, true, true, true)
            .command(DmMunmuteCommand, true, true, true)

            .command(NodmWarnCommand, true, true, false)
            .command(NodmMuteCommand, true, true, false)
            .command(NodmUnmuteCommand, true, true, false)
            .command(NodmBanCommand, true, true, false)
            .command(NodmKickCommand, true, true, false)
            .command(NodmMkickCommand, true, true, false)
            .command(NodmCleanBanCommand, true, true, false)
            .command(NodmUnbanCommand, true, true, false)
            .command(NodmTempmuteCommand, true, true, false)
            .command(NodmMmuteCommand, true, true, false)
            .command(NodmMwarnCommand, true, true, false)
            .command(NodmMbanCommand, true, true, false)
            .command(NodmTempbanCommand, true, true, false)
            .command(NodmMunbanCommand, true, true, false)
            .command(NodmMunmuteCommand, true, true, false)

            .command(SelfmuteCommand, true, true, false)

            // Owneronly Command Module
            .command(RefreshWhitelistRolesCommand, true, true, false)
            .command(RestartCommand, true, true, false)
            .command(StopCommand, true, true, false)
            .command(SupportinfoCommand, true, true, false)
            .command(SupportAnnounceCommand, true, true, false)

            // Utilities Command Module
            .command(CoinflipCommand, true, true, false)
            .command(EmojiCommand, true, true, false)
            .command(RandintCommand, true, true, false)

            // Whitelist Command Module
            .command(AcceptCommand, true, true, false)

            // Builds the parser with the configured commands and case sensitivity
            .build_parser()
    };

    let resource_types =
        ResourceType::CHANNEL |
        ResourceType::EMOJI |
        ResourceType::GUILD |
        ResourceType::MEMBER |
        ResourceType::MESSAGE |
        ResourceType::PRESENCE |
        ResourceType::REACTION |
        ResourceType::ROLE |
        ResourceType::USER |
        ResourceType::USER_CURRENT;

    let event_types =
        EventTypeFlags::BAN_ADD |
        EventTypeFlags::BAN_REMOVE |
        EventTypeFlags::CHANNEL_CREATE |
        EventTypeFlags::CHANNEL_DELETE |
        EventTypeFlags::CHANNEL_PINS_UPDATE |
        EventTypeFlags::CHANNEL_UPDATE |
        EventTypeFlags::GUILD_CREATE |
        EventTypeFlags::GUILD_DELETE |
        EventTypeFlags::GUILD_EMOJIS_UPDATE |
        EventTypeFlags::GUILD_INTEGRATIONS_UPDATE |
        EventTypeFlags::GUILD_UPDATE |
        EventTypeFlags::INVITE_CREATE |
        EventTypeFlags::INVITE_DELETE |
        EventTypeFlags::MEMBER_ADD |
        EventTypeFlags::MEMBER_CHUNK |
        EventTypeFlags::MEMBER_REMOVE |
        EventTypeFlags::MEMBER_UPDATE |
        EventTypeFlags::MESSAGE_CREATE |
        EventTypeFlags::MESSAGE_DELETE |
        EventTypeFlags::MESSAGE_DELETE_BULK |
        EventTypeFlags::MESSAGE_UPDATE |
        EventTypeFlags::READY |
        EventTypeFlags::SHARD_CONNECTED |
        EventTypeFlags::SHARD_CONNECTING |
        EventTypeFlags::SHARD_DISCONNECTED |
        EventTypeFlags::SHARD_IDENTIFYING |
        EventTypeFlags::SHARD_RECONNECTING |
        EventTypeFlags::USER_UPDATE |
        EventTypeFlags::VOICE_STATE_UPDATE |
        EventTypeFlags::WEBHOOKS_UPDATE;

    // Registering events
    let hartex_cache = InMemoryCache::builder()
        .resource_types(resource_types)
        .build();

    Logger::log_debug("Registered events.");

    // Framework Listeners
    let listeners = framework.clone().listeners();
    let emitter = CommandEventEmitter::new(listeners);

    // Cluster events
    let mut events = hartex_cluster.some_events(event_types);
    let mut command_events = framework.events();

    // Sets the Ctrl+C handler.
    ctrlc::set_handler(|| {
        Logger::log_warning("Received a Ctrl-C signal; terminating process.");

        std::process::exit(0);
    })?;

    // Start an event loop to process each event in the stream as they come in.
    while let value = futures_util::future::select(
        StreamExt::next(&mut events), command_events.next()).await {
        match value {
            Either::Left(event) => {
                hartex_cache.update(&event.0.clone().unwrap().1);

                tokio::spawn(
                    handle_event(
                        Some(event.0.clone().unwrap().0),
                        EventType::TwilightEvent,
                        Some(event.0.unwrap().1),
                        None,
                        hartex_http.clone(),
                        hartex_cluster.clone(),
                        command_parser.clone(),
                        hartex_cache.clone(),
                        stopwatch,
                        emitter.clone()
                    )
                );
            },
            Either::Right(event) => {
                tokio::spawn(
                    handle_event(
                        None,
                        EventType::CustomEvent,
                        None,
                        event.0,
                        hartex_http.clone(),
                        hartex_cluster.clone(),
                        command_parser.clone(),
                        hartex_cache.clone(),
                        stopwatch,
                        emitter.clone()
                    )
                );
            }
        }
    }

    Ok(())
}

async fn handle_event(_shard_id: Option<u64>,
                      event_type: EventType,
                      event: Option<Event>,
                      custom_event: Option<SystemEvent>,
                      http_client: TwilightHttpClient,
                      cluster: Cluster,
                      parser: CommandParser<'static>,
                      cache: InMemoryCache,
                      stopwatch: Stopwatch,
                      emitter: CommandEventEmitter)
    -> Result<(), Box<dyn Error + Send + Sync>> {
    match event_type {
        EventType::TwilightEvent => {
            if let Some(event) = event {
                match event {
                    Event::ShardConnecting(connecting) => {
                        EventHandler::shard_connecting(connecting).await
                    },
                    Event::ShardConnected(connected) => {
                        EventHandler::shard_connected(connected).await
                    },
                    Event::ShardIdentifying(identifying) => {
                        EventHandler::shard_identifying(identifying).await
                    },
                    Event::ShardReconnecting(reconnecting) => {
                        EventHandler::shard_reconnecting(reconnecting).await
                    },
                    Event::ShardDisconnected(disconnected) => {
                        EventHandler::shard_disconnected(disconnected).await
                    },
                    Event::Ready(ready) => {
                        EventHandler::ready(ready, stopwatch).await
                    },
                    Event::MessageCreate(message_create) => {
                        if (*message_create).author.bot {
                            return Ok(());
                        }

                        if message_create.content.starts_with("hb.") {
                            emitter.event(SystemEvent::CommandReceived(box CommandReceived));

                            match handle_command(
                                (*message_create).clone().0,
                                CommandContext(
                                    Arc::new(
                                        CommandContextRef::new(
                                            http_client.clone(),
                                            parser,
                                            cluster,
                                            (*message_create).clone().0,
                                            stopwatch
                                        )
                                    ),
                                ),
                                http_client.clone(),
                                cache,
                                emitter
                            ).await {
                                Ok(()) => (),
                                Err(error) => {
                                    Logger::log_error(format!("Failed to handle message: {}", error));

                                    let error_hash =
                                        format!("{:x}",
                                                Sha3_512::digest(format!("{}{:?}{:?}",
                                                                         error.to_string(),
                                                                         message_create.guild_id,
                                                                         message_create.id).as_str().as_bytes()));
                                    let message =
                                        format!(
                                            "Oops! This command raised an error. Please join or go to our Support Server (you need to get the **_HarTex** role, go to <#667597397215674368>) and provide the error code below for further troubleshooting and investigation.\n\nServer Invite: discord.gg/s8qjxZK\n\nError code: `{}`", error_hash);

                                    http_client.clone().create_message(message_create.channel_id).content(message)?.await?;
                                }
                            }
                        }
                        else if message_create.content.to_lowercase().contains("harry") {
                            http_client
                                .clone()
                                .create_reaction(message_create.channel_id,
                                                 message_create.id, RequestReactionType::Custom {
                                        id: EmojiId(683744109550108704),
                                        name: None
                                    }).await?;
                        }
                        else {}

                        Ok(())
                    },
                    Event::GuildCreate(guild_create) => {
                        EventHandler::guild_create(guild_create, http_client).await
                    },
                    _ => Ok(())
                }
            }
            else {
                return Err(box SystemError("EventType is TwilightEvent but event is None.".to_string()));
            }
        },
        EventType::CustomEvent => {
            if let Some(custom_event) = custom_event {
                match custom_event {
                    SystemEvent::CommandExecuted(command_executed) => {
                        EventHandler::command_executed(command_executed).await
                    },
                    SystemEvent::CommandFailed(command_failed) => {
                        EventHandler::command_failed(command_failed).await
                    }
                    SystemEvent::CommandReceived(command_received) => {
                        EventHandler::command_received(command_received).await
                    },
                    SystemEvent::CommandIdentified(command_identified) => {
                        EventHandler::command_identified(command_identified).await
                    }
                }
            }
            else {
                return Err(box SystemError("EventType is CustomEvent but custom_event is None.".to_string()));
            }
        }
    }?;

    Ok(())
}

async fn handle_command(message: Message,
                        context: CommandContext<'static>,
                        http_client: TwilightHttpClient,
                        cache: InMemoryCache,
                        emitter: CommandEventEmitter) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(command) = context.command_parser.parse(&message.content) {
        emitter.event(SystemEvent::CommandIdentified(command.name.to_string()));

        match command {
            // Administrator Command Module
            Command { name: "clean", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("all") => {
                        match CleanAllCommand::precommand_check(context.clone(),
                                                               PrecommandCheckParametersBuilder::new()
                                                                   .in_memory_cache(cache.clone())
                                                                   .minimum_permission_level(60).build(),
                                                               |ctx, params|
                                                                   Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match CleanAllCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "clean all",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "clean all",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "clean all",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("user") => {
                        match CleanUserCommand::precommand_check(context.clone(), 
                                                                PrecommandCheckParametersBuilder::new()
                                                                    .in_memory_cache(cache.clone())
                                                                    .minimum_permission_level(60).build(),
                                                                |ctx, params| 
                                                                    Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match CleanUserCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "clean user",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "clean user",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "clean user",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("bots") => {
                        match CleanBotsCommand::precommand_check(context.clone(),
                                                                 PrecommandCheckParametersBuilder::new()
                                                                     .in_memory_cache(cache.clone())
                                                                     .minimum_permission_level(60).build(),
                                                                 |ctx, params|
                                                                     Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match CleanUserCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "clean bots",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "clean bots",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "clean bots",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "role", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("add") => {
                        match RoleAddCommand::precommand_check(context.clone(),
                                                         PrecommandCheckParametersBuilder::new()
                                                             .in_memory_cache(cache.clone())
                                                             .minimum_permission_level(60).build(),
                                                         |ctx, params|
                                                             Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match RoleAddCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "role add",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "role add",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "role add",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("remove") => {
                        match RoleRemoveCommand::precommand_check(context.clone(),
                                                               PrecommandCheckParametersBuilder::new()
                                                                   .in_memory_cache(cache.clone())
                                                                   .minimum_permission_level(60).build(),
                                                               |ctx, params|
                                                                   Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match RoleRemoveCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "role remove",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "role remove",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "role remove",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("global-add") => {
                        match RoleGlobalAddCommand::precommand_check(context.clone(),
                                                                     PrecommandCheckParametersBuilder::new()
                                                                         .in_memory_cache(cache.clone())
                                                                         .minimum_permission_level(80).build(),
                                                                     |ctx, params|
                                                                         Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match RoleGlobalAddCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "role global-add",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "role gloabl-add",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "role global-add",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("global-remove") => {
                        match RoleGlobalRemoveCommand::precommand_check(context.clone(),
                                                                     PrecommandCheckParametersBuilder::new()
                                                                         .in_memory_cache(cache.clone())
                                                                         .minimum_permission_level(80).build(),
                                                                     |ctx, params|
                                                                         Box::pin(HasRolePermissions::execute_check(ctx, params)))
                            .await {
                            Ok(()) => {
                                match RoleGlobalRemoveCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "role global-remove",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "role global-remove",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "role global-remove",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "role-info", arguments, .. } => {
                match RoleinfoCommand::precommand_check(context.clone(),
                                                        PrecommandCheckParametersBuilder::new()
                                                            .in_memory_cache(cache.clone())
                                                            .minimum_permission_level(80).build(),
                                                        |ctx, params|
                                                            Box::pin(HasRolePermissions::execute_check(ctx, params)))
                    .await {
                    Ok(()) => {
                        match RoleinfoCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "role-info",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "role-info",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "role-info",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "lockdown", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("channel") => {
                        match LockdownChannelCommand::precommand_checks(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            Box::<[
                                    for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                    (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                |ctx, params|
                                    HasRolePermissions::execute_check(ctx, params)
                                , |ctx, params|
                                    GuildTextChannelOnly::execute_check(ctx, params)
                            ])).await {
                            Ok(()) => {
                                match LockdownChannelCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "lockdown channel",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "lockdown channel",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "lockdown channel",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("guild") => {
                        match LockdownGuildCommand::precommand_checks(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            Box::<[
                                    for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                    (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                |ctx, params|
                                    HasRolePermissions::execute_check(ctx, params)
                                , |ctx, params|
                                    GuildTextChannelOnly::execute_check(ctx, params)
                            ])).await {
                            Ok(()) => {
                                match LockdownGuildCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "lockdown guild",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "lockdown guild",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "lockdown guild",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "unlockdown", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("channel") => {
                        match UnlockdownChannelCommand::precommand_checks(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            Box::<[
                                    for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                    (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                |ctx, params|
                                    HasRolePermissions::execute_check(ctx, params)
                                , |ctx, params|
                                    GuildTextChannelOnly::execute_check(ctx, params)
                            ])).await {
                            Ok(()) => {
                                match UnlockdownChannelCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "unlockdown channel",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "unlockdowm channel",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "unlockdown channel",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("guild") => {
                        match UnlockdownGuildCommand::precommand_checks(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            Box::<[
                                    for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                    (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                |ctx, params|
                                    HasRolePermissions::execute_check(ctx, params)
                                , |ctx, params|
                                    GuildTextChannelOnly::execute_check(ctx, params)
                            ])).await {
                            Ok(()) => {
                                match UnlockdownGuildCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "unlockdown guild",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "unlockdown guild",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "unlockdown guild",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "slowmode", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("enable") => {
                        let position = arguments.next();

                        match position {
                            Some("here") => {
                                match SlowmodeEnableHereCommand::precommand_checks(
                                    context.clone(),
                                    PrecommandCheckParametersBuilder::new()
                                        .in_memory_cache(cache.clone())
                                        .minimum_permission_level(60).build(),
                                    Box::<[
                                            for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                        -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                        |ctx, params|
                                            HasRolePermissions::execute_check(ctx, params)
                                        , |ctx, params|
                                            GuildTextChannelOnly::execute_check(ctx, params)
                                    ])).await {
                                    Ok(()) => {
                                        match SlowmodeEnableHereCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                            Ok(()) => {
                                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                                    Some(guild) => guild.name,
                                                    None => String::new()
                                                };

                                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                                    command: "slowmode enable here",
                                                    guild_name: guild,
                                                    context: context.clone()
                                                }))
                                            },
                                            Err(error) => {
                                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                                    command: "slowmode enable here",
                                                    error: format!("{}", error)
                                                }))
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "slowmode enable here",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Some("channel") => {
                                match SlowmodeEnableChannelCommand::precommand_checks(
                                    context.clone(),
                                    PrecommandCheckParametersBuilder::new()
                                        .in_memory_cache(cache.clone())
                                        .minimum_permission_level(60).build(),
                                    Box::<[
                                            for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                        -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                        |ctx, params|
                                            HasRolePermissions::execute_check(ctx, params)
                                        , |ctx, params|
                                            GuildTextChannelOnly::execute_check(ctx, params)
                                    ])).await {
                                    Ok(()) => {
                                        match SlowmodeEnableChannelCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                            Ok(()) => {
                                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                                    Some(guild) => guild.name,
                                                    None => String::new()
                                                };

                                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                                    command: "slowmode enable channel",
                                                    guild_name: guild,
                                                    context: context.clone()
                                                }))
                                            },
                                            Err(error) => {
                                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                                    command: "slowmode enable channel",
                                                    error: format!("{}", error)
                                                }))
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "slowmode enable channel",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            _ => Logger::log_error(
                                format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                        }
                    },
                    Some("disable") => {
                        let position = arguments.next();

                        match position {
                            Some("here") => {
                                match SlowmodeDisableHereCommand::precommand_checks(
                                    context.clone(),
                                    PrecommandCheckParametersBuilder::new()
                                        .in_memory_cache(cache.clone())
                                        .minimum_permission_level(60).build(),
                                    Box::<[
                                            for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                        -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                        |ctx, params|
                                            HasRolePermissions::execute_check(ctx, params)
                                        , |ctx, params|
                                            GuildTextChannelOnly::execute_check(ctx, params)
                                    ])).await {
                                    Ok(()) => {
                                        match SlowmodeDisableHereCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                            Ok(()) => {
                                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                                    Some(guild) => guild.name,
                                                    None => String::new()
                                                };

                                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                                    command: "slowmode disable here",
                                                    guild_name: guild,
                                                    context: context.clone()
                                                }))
                                            },
                                            Err(error) => {
                                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                                    command: "slowmode disable here",
                                                    error: format!("{}", error)
                                                }))
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "slowmode disable here",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Some("channel") => {
                                match SlowmodeDisableChannelCommand::precommand_checks(
                                    context.clone(),
                                    PrecommandCheckParametersBuilder::new()
                                        .in_memory_cache(cache.clone())
                                        .minimum_permission_level(60).build(),
                                    Box::<[
                                            for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                                                        -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                                                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                                        |ctx, params|
                                            HasRolePermissions::execute_check(ctx, params)
                                        , |ctx, params|
                                            GuildTextChannelOnly::execute_check(ctx, params)
                                    ])).await {
                                    Ok(()) => {
                                        match SlowmodeDisableChannelCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                            Ok(()) => {
                                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                                    Some(guild) => guild.name,
                                                    None => String::new()
                                                };

                                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                                    command: "slowmode disable channel",
                                                    guild_name: guild,
                                                    context: context.clone()
                                                }))
                                            },
                                            Err(error) => {
                                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                                    command: "slowmode disable channel",
                                                    error: format!("{}", error)
                                                }))
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "slowmode disable channel",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            _ => Logger::log_error(
                                format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "voicemute", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("enable") => {
                        match VoicemuteEnableCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match VoicemuteEnableCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "voicemute enable",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "voicemute enable",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "voicemut enable",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("disable") => {
                        match VoicemuteDisableCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match VoicemuteDisableCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "voicemute disable",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "voicemute disable",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "voicemute disable",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "noroles", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("list") => {
                        match NorolesListCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match NorolesListCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "noroles list",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "noroles list",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "noroles list",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("kick") => {
                        match NorolesKickCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(80).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match NorolesKickCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "noroles kick",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "noroles kick",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "noroles kick",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "nickname", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("change") => {
                        match NicknameChangeCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match NicknameChangeCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "nickname change",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "nickname change",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nickname change",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("remove") => {
                        match NicknameRemoveCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                HasRolePermissions::execute_check(ctx, params)).await {
                            Ok(()) => {
                                match NicknameRemoveCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "nickname remove",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "nickname remove",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nickname remove",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "webconfig", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("list") => {
                        match WebconfigListCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(80).build(),
                                |ctx, params|
                                    HasRolePermissions::execute_check(ctx, params)
                        ).await {
                            Ok(()) => {
                                match WebconfigListCommand::execute_command(context.clone(), arguments, cache.clone()).await {
                                    Ok(()) => {
                                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                            Some(guild) => guild.name,
                                            None => String::new()
                                        };

                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "webconfig list",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "webconfig list",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "webconfig list",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => Logger::log_error(
                        format!("Command '{}' failed due to an error: 'command not found'.", message.content))
                }
            },
            Command { name: "invites", .. } => {
                match InvitesCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(80).build(),
                        |ctx, params| 
                            HasRolePermissions::execute_check(ctx, params)
                ).await {
                    Ok(()) => {

                    },
                    Err(_error) => {
                        
                    }
                }
            },

            // General Command Module
            Command { name: "ping", arguments, .. } => {
                match PingCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "ping",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "ping",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "bot-info", arguments, .. } => {
                match BotinfoCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "bot-info",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "bot-info",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "help", arguments, .. } => {
                match HelpCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "help",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "help",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "about", arguments, .. } => {
                match AboutCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "about",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "about",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "team", arguments, .. } | Command { name: "staff", arguments, .. } => {
                match TeamCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "team",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "team",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "uptime", arguments, .. } => {
                match UptimeCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "uptime",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "uptime",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Information Command Module
            Command { name: "user-info", arguments, .. } => {
                match UserinfoCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "user-info",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "user-info",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "guild-info", arguments, .. } | Command { name: "server-info", arguments, .. } => {
                match GuildinfoCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "guild-info",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "guild-info",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Guild Owneronly Command Module
            Command { name: "setup", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match SetupCommand::precommand_checks(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .guild_id(context.clone().message.guild_id.unwrap()).build(),
                    Box::<[
                            for<'asynchronous_trait> fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                                -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                                    (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                        |ctx, params|
                            GuildOwnerOnly::execute_check(ctx, params)
                        , |ctx, params|
                            GuildIsAlreadySetup::execute_check(ctx, params)
                    ])).await {
                    Ok(()) => {
                        match SetupCommand::execute_command(context.clone(), arguments, cache)
                            .await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "setup",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "setup",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "setup",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Infractions Command Module
            Command { name: "warn", arguments, .. }  |
            Command { name: "dmwarn", arguments, .. } => {
                match DmWarnCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        match DmWarnCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmwarn",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmwarn",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmwarn",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmwarn", arguments, .. } => {
                match NodmWarnCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        match NodmWarnCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmwarn",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmwarn",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmwarn",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "inf", mut arguments, .. } => {
                let subcommand = arguments.next();

                match subcommand {
                    Some("search") => {
                        match InfractionSearchCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(80).build(),
                            |ctx, params|
                                Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                match InfractionSearchCommand::execute_command(context.clone(), arguments, cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "inf search",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "inf search",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "inf search",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("remove") => {
                        match InfractionRemoveCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(80).build(),
                            |ctx, params|
                                Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                match InfractionRemoveCommand::execute_command(context.clone(), arguments, cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "inf remove",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "inf remove",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "inf remove",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("archive") => {
                        match InfractionsArchiveCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(80).build(),
                            |ctx, params|
                                Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                match InfractionsArchiveCommand::execute_command(context.clone(), arguments, cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "inf archive",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "inf archive",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "inf archive",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("clear-all") => {
                        match InfractionClearallCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                match InfractionClearallCommand::execute_command(context.clone(), arguments, cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "inf clear-all",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "inf clear-all",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "inf clear-all",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Some("reason") => {
                        match InfractionReasonCommand::precommand_check(
                            context.clone(),
                            PrecommandCheckParametersBuilder::new()
                                .in_memory_cache(cache.clone())
                                .minimum_permission_level(60).build(),
                            |ctx, params|
                                Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                match InfractionReasonCommand::execute_command(context.clone(), arguments, cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "inf reason",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "inf reason",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "inf reason",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "unknown",
                            error: String::from("command not found.")
                        }))
                    }
                }
            },
            Command { name: "mute", arguments, .. } |
            Command { name: "dmmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "unmute", arguments, .. } |
            Command { name: "dmunmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmUnmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmUnmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmunmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmunmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmunmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmunmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmUnmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmUnmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmunmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmunmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmunmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "kick", arguments, .. } |
            Command { name: "dmkick", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmKickCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmKickCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmkick",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmkick",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmkick",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmkick", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmKickCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmKickCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmkick",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmkick",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmkick",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "ban", arguments, .. } |
            Command { name: "dmban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmBanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmBanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmBanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmBanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "mkick", arguments, .. } |
            Command { name: "dmmkick", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMkickCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMkickCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmkick",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmkick",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmkick",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmkick", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMkickCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMkickCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmkick",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmkick",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmkick",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "cleanban", arguments, .. } |
            Command { name: "dmcleanban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmCleanBanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmCleanBanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmcleanban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmcleanban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmcleanban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmcleanban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmCleanBanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmCleanBanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmcleanban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmcleanban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmcleanban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "unban", arguments, .. } |
            Command { name: "dmunban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmUnbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmUnbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmunban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmunban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmunban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmunban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmUnbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmUnbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmunban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmunban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmunban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "tempmute", arguments, .. } |
            Command { name: "dmtempmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmTempmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmTempmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmtempmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmtempmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "tempmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmtempmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmTempmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmTempmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmtempmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmtempmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmtempmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "mmute", arguments, .. } |
            Command { name: "dmmmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "munmute", arguments, .. } |
            Command { name: "dmunmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMunmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMunmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmunmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMunmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMunmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmunmute", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMunmuteCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMunmuteCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmunmute",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmunmute",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmunmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "mwarn", arguments, .. } |
            Command { name: "dmmwarn", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMwarnCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMwarnCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmwarn",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmwarn",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmwarn",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmwarn", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMwarnCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMwarnCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmwarn",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmwarn",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmwarn",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "mban", arguments, .. }|
            Command { name: "dmmban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "selfmute", arguments, .. } => {
                match SelfmuteCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "selfmute",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "selfmute",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "munban", arguments, .. }|
            Command { name: "dmmunban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match DmMunbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match DmMbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "dmmunban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "dmmunban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "dmmunban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "nodmmunban", arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                match NodmMunbanCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new()
                        .in_memory_cache(cache.clone())
                        .minimum_permission_level(60).build(),
                    |ctx, params|
                        Box::pin(HasRolePermissions::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match NodmMbanCommand::execute_command(context.clone(), arguments, cache).await {
                            Ok(()) => {
                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "nodmmunban",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "nodmmunban",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "nodmmunban",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Owneronly Command Module
            Command { name: "restart", arguments, .. } => {
                match RestartCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new().build(),
                    |ctx, params|
                        Box::pin(BotOwnerOnly::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match RestartCommand::execute_command(context.clone(), arguments, cache)
                            .await {
                            Ok(()) => {
                                todo!()
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "restart",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "restart",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "stop", arguments, .. } => {
                match StopCommand::precommand_check(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new().build(),
                    |ctx, params|
                        Box::pin(BotOwnerOnly::execute_check(ctx, params))).await {
                    Ok(()) => {
                        match StopCommand::execute_command(
                            context.clone(), arguments, cache).await {
                            Ok(()) => {
                                Logger::log_info("Bot stopping...");
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "stop",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "stop",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "support-info", arguments, .. } => {
                match SupportinfoCommand::precommand_checks(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new().build(),
                    Box::<[for<'asynchronous_trait>
                    fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                        -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                        |ctx, params| BotOwnerOnly::execute_check(ctx, params),
                        |ctx, params| SupportGuildOnly::execute_check(ctx, params)
                    ])).await {
                    Ok(()) => {
                        match SupportinfoCommand::execute_command(context.clone(), arguments, cache)
                            .await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "support-info",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "support-info",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "support-info",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "refresh-whitelist-roles", arguments, .. } => {
                match RefreshWhitelistRolesCommand::precommand_checks(
                    context.clone(), PrecommandCheckParametersBuilder::new().build(),
                    Box::<[for<'asynchronous_trait>
                    fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                        -> Pin<Box<dyn std::future::Future<Output =std::result::Result<
                            (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>
                    ::new([
                        |ctx, params| BotOwnerOnly::execute_check(ctx, params),
                        |ctx, params| SupportGuildOnly::execute_check(ctx, params)
                    ])).await {
                    Ok(()) => {
                        match RefreshWhitelistRolesCommand::execute_command(context.clone(), arguments,
                                                                            cache).await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "refresh-whitelist-roles",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "refresh-whitelist-roles",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "refresh-whitelist-roles",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "support-announce", arguments, .. } => {
                match SupportAnnounceCommand::precommand_checks(
                    context.clone(),
                    PrecommandCheckParametersBuilder::new().build(),
                    Box::<[for<'asynchronous_trait>
                    fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                       -> Pin<Box<dyn std::future::Future<Output = std::result::Result<
                           (), Box<(dyn Error + Send + Sync)>>> + Send + 'asynchronous_trait>>; 2]>::new([
                        |ctx, params| BotOwnerOnly::execute_check(ctx, params),
                        |ctx, params| SupportGuildOnly::execute_check(ctx, params)
                    ])).await {
                    Ok(()) => {
                        match SupportAnnounceCommand::execute_command(context.clone(), arguments, cache)
                            .await {
                            Ok(()) => {
                                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                                    Some(guild) => guild.name,
                                    None => String::new()
                                };

                                emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                    command: "support-announce",
                                    guild_name: guild,
                                    context: context.clone()
                                }))
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "support-announce",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "support-announce",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Utilities Command Module
            Command { name: "emoji", arguments, .. } => {
                match EmojiCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "emoji",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "emoji",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "randint", arguments, .. } => {
                match RandintCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "randint",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "randint",
                            error: format!("{}", error)
                        }))
                    }
                }
            },
            Command { name: "coinflip", arguments, .. } => {
                match CoinflipCommand::execute_command(context.clone(), arguments, cache).await {
                    Ok(()) => {
                        let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                            Some(guild) => guild.name,
                            None => String::new()
                        };

                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                            command: "coinflip",
                            guild_name: guild,
                            context: context.clone()
                        }))
                    },
                    Err(error) => {
                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                            command: "coinflip",
                            error: format!("{}", error)
                        }))
                    }
                }
            },

            // Whitelist Command Module
            Command { name: "whitelist", mut arguments, .. } => {
                let guild = match http_client.guild(message.guild_id.unwrap()).await? {
                    Some(guild) => guild.name,
                    None => String::new()
                };

                let subcommand = arguments.next();

                match subcommand {
                    Some("accept") => {
                        match AcceptCommand::precommand_check(
                            context.clone(), PrecommandCheckParametersBuilder::new().build(),
                            |ctx, params|
                                Box::pin(BotOwnerOnly::execute_check(ctx, params))).await {
                            Ok(()) => {
                                match AcceptCommand::execute_command(context.clone(), arguments,
                                                                     cache).await {
                                    Ok(()) => {
                                        emitter.event(SystemEvent::CommandExecuted(box CommandExecuted {
                                            command: "whitelist accept",
                                            guild_name: guild,
                                            context: context.clone()
                                        }))
                                    },
                                    Err(error) => {
                                        emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                            command: "whitelist accept",
                                            error: format!("{}", error)
                                        }))
                                    }
                                }
                            },
                            Err(error) => {
                                emitter.event(SystemEvent::CommandFailed(box CommandFailed {
                                    command: "whitelist accept",
                                    error: format!("{}", error)
                                }))
                            }
                        }
                    },
                    _ => {
                        Logger::log_error(
                            format!(
                                "Command '{}' failed due to an error: 'command not found'.", message.content
                            )
                        );
                    }
                }
            },

            _ => ()
        }
    }
    else {
    }

    Ok(())
}
