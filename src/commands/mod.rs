pub mod minecraft;

use crate::{Context, Result};

/// Display the invite link to the LOTR Mod Community discord
#[poise::command(slash_command)]
pub async fn discord(ctx: Context<'_>) -> Result {
    ctx.say(
        "The invite for the LOTR Mod Community Discord is available here:
https://discord.gg/QXkZzKU",
    )
    .await?;
    Ok(())
}
