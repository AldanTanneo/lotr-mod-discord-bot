use crate::serenity;
use crate::{Context, Result};
use sqlx::FromRow;

pub async fn is_admin(
    ctx: &Context<'_>,
    user_id: serenity::UserId,
    guild_id: serenity::GuildId,
) -> Result<bool> {
    let (is_admin,) = sqlx::query_as::<_, (bool,)>(
        "SELECT EXISTS(SELECT perm_id FROM bot_admins WHERE server_id = ? AND user_id = ? LIMIT 1)",
    )
    .bind(guild_id.0)
    .bind(user_id.0)
    .fetch_one(&ctx.data().db_pool)
    .await?;

    Ok(is_admin)
}

pub async fn get_admins(
    ctx: &Context<'_>,
    guild_id: serenity::GuildId,
) -> Result<Vec<serenity::UserId>> {
    let admins = sqlx::query("SELECT user_id FROM bot_admins WHERE server_id = ?")
        .bind(guild_id.0)
        .try_map(|row| FromRow::from_row(&row).map(|(id,)| serenity::UserId(id)))
        .fetch_all(&ctx.data().db_pool)
        .await?;

    Ok(admins)
}
