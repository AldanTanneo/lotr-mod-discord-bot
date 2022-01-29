#[cfg(not(feature = "noperms"))]
use crate::constants::MANAGE_BOT_PERMS;
use crate::database;
use crate::{Context, Result};

pub async fn is_admin(ctx: Context<'_>) -> Result<bool> {
    let user_id = ctx.author().id;
    #[cfg(not(feature = "noperms"))]
    if ctx.framework().options().owners.contains(&user_id) {
        // Owner privileges
        return Ok(true);
    }

    let guild_id = if let Some(guild_id) = ctx.guild_id() {
        #[cfg(not(feature = "noperms"))]
        if guild_id
            .member(ctx.discord(), user_id)
            .await?
            .permissions(ctx.discord())
            .map(|p| p.intersects(MANAGE_BOT_PERMS))
            .unwrap_or_default()
        {
            // The MANAGE_BOT_PERMS permission set bypasses the db admin check
            return Ok(true);
        }
        guild_id
    } else {
        // No one is an admin outside of a guild
        ctx.defer_ephemeral().await?;
        ctx.say(":x: This command cannot be executed in DMs!")
            .await?;
        return Ok(false);
    };

    let is_admin = database::admin::is_admin(&ctx, user_id, guild_id)
        .await
        .unwrap_or_default();

    if !is_admin {
        ctx.defer_ephemeral().await?;
        ctx.say(":x: You are not an admin on this server!").await?;
    }

    Ok(is_admin)
}

pub async fn is_guild(ctx: Context<'_>) -> Result<bool> {
    let is_guild = ctx.guild_id().is_some();

    if !is_guild {
        ctx.defer_ephemeral().await?;
        ctx.say(":x: You can only use this in a guild!").await?;
    }

    Ok(is_guild)
}
