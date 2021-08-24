use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;

use crate::constants::TABLE_ADMINS;
use crate::get_database_conn;

pub async fn is_admin_function(ctx: &Context, server_id: GuildId, user: UserId) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    let res = conn
        .query_first(format!(
            "SELECT EXISTS(SELECT perm_id FROM {} WHERE server_id={} AND user_id={} LIMIT 1)",
            TABLE_ADMINS, server_id.0, user.0
        ))
        .await
        .ok()?;

    res
}

pub async fn get_admins(ctx: &Context, server_id: GuildId) -> Option<Vec<UserId>> {
    let mut conn = get_database_conn!(ctx);

    let res = conn
        .exec_map(
            format!(
                "SELECT user_id FROM {} WHERE server_id=:server_id",
                TABLE_ADMINS
            )
            .as_str(),
            params! {
                "server_id" => server_id.0
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
    server_id: GuildId,
    user_id: UserId,
    floppadmin: bool,
) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        format!(
        "REPLACE INTO {} (server_id, user_id, floppadmin) VALUES (:server_id, :user_id, :floppa)",
        TABLE_ADMINS
    ),
        params! {
            "server_id" => server_id.0,
            "user_id" => user_id.0,
            "floppa" => floppadmin,
        },
    )
    .await?;

    Ok(())
}

pub async fn remove_admin(ctx: &Context, server_id: GuildId, user_id: UserId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id LIMIT 1",
            TABLE_ADMINS
        )
        .as_str(),
        params! {
            "server_id" => server_id.0,
            "user_id" => user_id.0,
        },
    )
    .await?;

    drop(conn);

    Ok(())
}
