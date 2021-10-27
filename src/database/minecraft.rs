use crate::{Context, Result};

pub async fn get_minecraft_ip(ctx: &Context<'_>) -> Result<String> {
    let guild_id = ctx.guild_id().ok_or("Not in a Guild")?.0;

    let (ip,) =
        sqlx::query_as::<_, (String,)>("SELECT mc_ip FROM mc_server_ip WHERE server_id = ?")
            .bind(guild_id)
            .fetch_one(&ctx.data().db_pool)
            .await?;

    Ok(ip)
}

pub async fn set_minecraft_ip(ctx: &Context<'_>, ip: &str) -> Result {
    let guild_id = ctx.guild_id().ok_or("Not in a Guild")?.0;

    sqlx::query("REPLACE INTO mc_server_ip (server_id, mc_ip) VALUES (?, ?)")
        .bind(guild_id)
        .bind(ip)
        .execute(&ctx.data().db_pool)
        .await?;

    Ok(())
}

pub async fn delete_minecraft_ip(ctx: &Context<'_>) -> Result {
    let guild_id = ctx.guild_id().ok_or("Not in a Guild")?.0;

    sqlx::query("DELETE FROM mc_server_ip WHERE server_id = ? LIMIT 1")
        .bind(guild_id)
        .execute(&ctx.data().db_pool)
        .await?;

    Ok(())
}
