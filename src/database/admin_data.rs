use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::error::Error::WrongGuild;
use serenity::model::prelude::*;

use crate::constants::TABLE_ADMINS;
use crate::get_database_conn;

pub async fn is_admin_function(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user: UserId,
) -> Option<bool> {
    let server_id: u64 = guild_id?.0;

    let mut conn = get_database_conn!(ctx);

    let res = conn
        .query_first(format!(
            "SELECT EXISTS(SELECT perm_id FROM {} WHERE server_id={} AND user_id={} LIMIT 1)",
            TABLE_ADMINS, server_id, user.0
        ))
        .await
        .ok()?;

    drop(conn);

    res
}

pub async fn get_admins(ctx: &Context, guild_id: Option<GuildId>) -> Option<Vec<UserId>> {
    let server_id: u64 = guild_id?.0;

    let mut conn = get_database_conn!(ctx);

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
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

    let mut conn = get_database_conn!(ctx, Result);

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
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

    let mut conn = get_database_conn!(ctx, Result);

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id LIMIT 1",
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
