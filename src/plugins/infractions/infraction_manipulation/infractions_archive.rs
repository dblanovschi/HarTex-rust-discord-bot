use std::{
    future::Future,
    fs::File,
    pin::Pin
};

use chrono::{
    Local
};

use csv::Writer;

use sha3::{
    Digest,
    Sha3_224
};

use twilight_cache_inmemory::InMemoryCache;

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
    twilight_http_client_extensions::GetGuildInfractions,
    twilight_id_extensions::IntoInnerU64,
    SystemResult
};

use crate::utilities::{
    FutureResult
};

use crate::xml_deserialization::BotConfig;
use std::io::Read;

crate struct InfractionsArchiveCommand;

impl Command for InfractionsArchiveCommand {
    fn name(&self) -> String {
        String::from("inf")
    }

    fn fully_qualified_name(&self) -> String {
        String::from("inf archive")
    }

    fn execute_command<'asynchronous_trait>(ctx: CommandContext<'asynchronous_trait>, _arguments: Arguments<'asynchronous_trait>, _cache: InMemoryCache)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(infractions_infractions_archive_command(ctx))
    }

    fn precommand_check<'asynchronous_trait, C>(ctx: CommandContext<'asynchronous_trait>, params: PrecommandCheckParameters, check: C)
        -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>>
        where
            C: Fn(CommandContext<'asynchronous_trait>, PrecommandCheckParameters)
                -> Pin<Box<dyn Future<Output=SystemResult<()>> + Send + 'asynchronous_trait>> {
        Box::pin(FutureResult::resolve(check(ctx, params)))
    }
}

async fn infractions_infractions_archive_command(ctx: CommandContext<'_>) -> SystemResult<()> {
    let guild_id = ctx.message.guild_id.unwrap();
    let now = Local::now().timestamp();
    let csv_file =
        File::create(
            format!(
                "csv/{}_guildinfs_{}.csv",
                now,
                guild_id.into_inner_u64()
            )
        )?;
    let mut writer = csv::Writer::from_writer(csv_file);
    let infraction_map = ctx
        .http_client
        .clone()
        .get_guild_infractions(ctx.message.guild_id.unwrap())
        .await?;

    writer.write_record(&["User ID", "Infraction ID", "Infraction Type", "Reason"])?;

    for (user_id, infractions) in infraction_map {
        for infraction in infractions {
            writer.write_record(&[
                &format!("{}", user_id.into_inner_u64()),
                &infraction.infraction_id,
                &format!("{}", infraction.infraction_type),
                &infraction.reason
            ])?;
        }
    }

    writer.flush()?;

    ctx.http_client
        .clone()
        .create_message(ctx.message.channel_id)
        .allowed_mentions()
        .replied_user(false)
        .build()
        .attachment(
            format!("{}_guildinfs_{}.csv", now, guild_id),
            std::fs::read(format!(
                "csv/{}_guildinfs_{}.csv",
                now,
                guild_id.into_inner_u64()
            ))?
        )
        .reply(ctx.message.id)
        .await?;

    std::fs::remove_file(format!(
        "csv/{}_guildinfs_{}.csv",
        now,
        guild_id.into_inner_u64()
    ))?;

    Ok(())
}
