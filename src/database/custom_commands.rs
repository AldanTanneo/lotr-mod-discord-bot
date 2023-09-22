use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::GuildId;

use crate::constants::TABLE_CUSTOM_COMMANDS;
use crate::get_database_conn;

#[derive(Debug, Clone)]
pub struct CustomCommand {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
}

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
    let mut conn = get_database_conn!(ctx);
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
            "INSERT INTO {TABLE_CUSTOM_COMMANDS} (server_id, name, command_json, documentation) VALUES (:server_id, :name, :body, :description)"
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
    let mut conn = get_database_conn!(ctx);

    let req = format!(
        "DELETE FROM {TABLE_CUSTOM_COMMANDS} WHERE server_id = :server_id AND name = :name LIMIT 1"
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
                "SELECT command_json FROM {TABLE_CUSTOM_COMMANDS} WHERE server_id = :server_id AND name = :name",
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
                "SELECT documentation FROM {TABLE_CUSTOM_COMMANDS} WHERE server_id = :server_id AND name = :name",
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
            "SELECT name, documentation FROM {TABLE_CUSTOM_COMMANDS} WHERE server_id = :server_id ORDER BY documentation DESC"
        )
        .as_str(),
        params! {
            "server_id" => server_id.0
        }
    )
    .await
    .ok()
}
