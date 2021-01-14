use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::{GuildId, UserId};

use super::DatabasePool;
use crate::constants::TABLE_ADMINS;

pub async fn get_admins(ctx: &Context, guild_id: Option<GuildId>) -> Option<Vec<UserId>> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;
    let server_id: u64 = guild_id?.0;

    let res = conn
        .exec_map(
            format!(
                "SELECT user_id FROM {} WHERE server_id=:server_id",
                TABLE_ADMINS
            )
            .as_str(),
            params! {
                "server_id" => server_id
            },
            UserId,
        )
        .await
        .ok()?;

    drop(conn);

    Some(res)
}

pub async fn add_admin(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
    update: bool,
    floppadmin: bool,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;
    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

    let req = if update {
        format!(
                "UPDATE {} SET floppadmin = :floppa WHERE server_id = :server_id AND user_id = :user_id",
                TABLE_ADMINS
            )
    } else {
        format!(
                "INSERT INTO {} (server_id, user_id, floppadmin) VALUES (:server_id, :user_id, :floppa)",
                TABLE_ADMINS
            )
    };

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "user_id" => user_id.0,
            "floppa" => floppadmin,
        },
    )
    .await?;

    Ok(())
}

pub async fn remove_admin(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id",
            TABLE_ADMINS
        )
        .as_str(),
        params! {
            "server_id" => server_id,
            "user_id" => user_id.0,
        },
    )
    .await?;

    drop(conn);

    Ok(())
}
