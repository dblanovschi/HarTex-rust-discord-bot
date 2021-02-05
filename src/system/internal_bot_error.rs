use std::{
    panic::{
        PanicInfo
    }
};

use crate::{
    command_system::CommandContext,
    system::SystemResult
};

crate const BOT_SUPPORT_SERVER: &str = "https://discord.gg/s8qjxZK";

crate async fn report_ibe(context: CommandContext<'_>, panic_info: &PanicInfo<'_>) -> SystemResult<()> {
    let channel_id = context.message.channel_id;

    Ok(())
}
