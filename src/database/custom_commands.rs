use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{error::Error::WrongGuild, id::GuildId};

use super::{CustomCommand, DatabasePool};
use crate::constants::TABLE_CUSTOM_COMMANDS;

pub async fn add_custom_command(
    ctx: &Context,
    guild_id: Option<GuildId>,
    name: &str,
    body: String,
    description: Option<&str>,
    update: bool,
) -> CommandResult {
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

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

    let req = if update {
        println!("updating...");
        format!(
            "UPDATE {} SET command_json = :body, documentation = :description WHERE server_id = :server_id AND name = :name",
            TABLE_CUSTOM_COMMANDS
        )
    } else {
        println!("adding...");
        format!(
            "INSERT INTO {} (server_id, name, command_json, documentation) VALUES (:server_id, :name, :body, :description)",
            TABLE_CUSTOM_COMMANDS
        )
    };

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "name" => name,
            "body" => body,
            "description" => description.unwrap_or_default()
        },
    )
    .await?;

    Ok(())
}

pub async fn remove_custom_command(
    ctx: &Context,
    guild_id: Option<GuildId>,
    name: &str,
) -> CommandResult {
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

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

    let req = format!(
        "DELETE FROM {} WHERE server_id = :server_id AND name = :name LIMIT 1",
        TABLE_CUSTOM_COMMANDS
    );

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "name" => name
        },
    )
    .await?;

    Ok(())
}

pub async fn get_command_data(
    ctx: &Context,
    guild_id: Option<GuildId>,
    name: &str,
    desc: bool,
) -> Option<CustomCommand> {
    let server_id: u64 = guild_id?.0;

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let body = conn
        .query_first(format!(
            "SELECT command_json FROM {} WHERE server_id={} AND name=\"{}\"",
            TABLE_CUSTOM_COMMANDS, server_id, name
        ))
        .await
        .ok()??;

    let description = if desc {
        conn.query_first(format!(
            "SELECT documentation FROM {} WHERE server_id={} AND name=\"{}\"",
            TABLE_CUSTOM_COMMANDS, server_id, name
        ))
        .await
        .ok()??
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
    guild_id: Option<GuildId>,
) -> Option<Vec<String>> {
    let server_id: u64 = guild_id?.0;

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    conn.exec(
        format!(
            "SELECT name FROM {} WHERE server_id=:server_id",
            TABLE_CUSTOM_COMMANDS
        )
        .as_str(),
        params! {
            "server_id" => server_id
        },
    )
    .await
    .ok()
}
