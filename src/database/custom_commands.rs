use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::GuildId;

use super::CustomCommand;
use crate::constants::TABLE_CUSTOM_COMMANDS;
use crate::get_database_conn;

pub async fn check_command_exists(ctx: &Context, server_id: GuildId, name: &str) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    conn.query_first(format!(
        "SELECT EXISTS(SELECT command_id FROM {} WHERE server_id={} AND name=\"{}\" LIMIT 1)",
        TABLE_CUSTOM_COMMANDS, server_id.0, name
    ))
    .await
    .ok()?
}

pub async fn add_custom_command(
    ctx: &Context,
    server_id: GuildId,
    name: &str,
    body: &str,
    description: Option<&str>,
) -> CommandResult {
    let mut conn = get_database_conn!(ctx, Result);
    let query = if check_command_exists(ctx, server_id, name)
        .await
        .unwrap_or_default()
    {
        format!(
            "UPDATE {} SET command_json = :body{} WHERE server_id = :server_id AND name = :name",
            TABLE_CUSTOM_COMMANDS,
            if description.is_some() {
                ", documentation = :description"
            } else {
                ""
            }
        )
    } else {
        format!(
            "INSERT INTO {} (server_id, name, command_json, documentation) VALUES (:server_id, :name, :body, :description)",
            TABLE_CUSTOM_COMMANDS
        )
    };

    conn.exec_drop(
        query,
        params! {
            "server_id" => server_id.0,
            "name" => name,
            "body" => body,
            "description" => description.unwrap_or_default()
        },
    )
    .await?;

    Ok(())
}

pub async fn remove_custom_command(ctx: &Context, server_id: GuildId, name: &str) -> CommandResult {
    let mut conn = get_database_conn!(ctx, Result);

    let req = format!(
        "DELETE FROM {} WHERE server_id = :server_id AND name = :name LIMIT 1",
        TABLE_CUSTOM_COMMANDS
    );

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id.0,
            "name" => name
        },
    )
    .await?;

    Ok(())
}

pub async fn get_command_data(
    ctx: &Context,
    server_id: GuildId,
    name: &str,
    desc: bool,
) -> Option<CustomCommand> {
    let mut conn = get_database_conn!(ctx);

    let body = conn
        .exec_first(
            format!(
                "SELECT command_json FROM {} WHERE server_id = :server_id AND name = :name",
                TABLE_CUSTOM_COMMANDS,
            ),
            params! {
                "server_id" => server_id.0,
                "name" => name
            },
        )
        .await
        .ok()??;

    let description = if desc {
        conn.exec_first(
            format!(
                "SELECT documentation FROM {} WHERE server_id = :server_id AND name = :name",
                TABLE_CUSTOM_COMMANDS,
            ),
            params! {
                "server_id" => server_id.0,
                "name" => name
            },
        )
        .await
        .ok()?
    } else {
        None
    };

    Some(CustomCommand {
        name: name.into(),
        body,
        description,
    })
}

pub async fn get_custom_commands_list(
    ctx: &Context,
    server_id: GuildId,
) -> Option<Vec<(String, String)>> {
    let mut conn = get_database_conn!(ctx);

    conn.exec(
        format!(
            "SELECT name, documentation FROM {} WHERE server_id = :server_id ORDER BY documentation DESC",
            TABLE_CUSTOM_COMMANDS
        )
        .as_str(),
        params! {
            "server_id" => server_id.0
        }
    )
    .await
    .ok()
}
