use dashmap::DashMap;
use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{error::Error::WrongGuild, id::GuildId};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use crate::constants::{TABLE_MC_SERVER_IP, TABLE_PREFIX};
use crate::get_database_conn;

pub struct PrefixCache;

impl TypeMapKey for PrefixCache {
    type Value = Arc<DashMap<GuildId, String>>;
}

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> Option<String> {
    let server_id = guild_id?;

    let prefix_cache = ctx.data.read().await.get::<PrefixCache>()?.clone();

    if let Some(prefix) = prefix_cache.get(&server_id) {
        return Some(prefix.value().clone());
    }

    let mut conn = get_database_conn!(ctx);

    let res: Option<String> = conn
        .query_first(format!(
            "SELECT prefix FROM {} WHERE server_id={}",
            TABLE_PREFIX, server_id.0
        ))
        .await
        .ok()?;

    let prefix = if let Some(prefix) = res {
        prefix
    } else {
        println!("Initializing prefix for {:?}", guild_id);
        set_prefix(ctx, guild_id, "!", false).await.ok()?;
        println!("Prefix initialized successfully");
        "!".to_string()
    };

    prefix_cache.insert(server_id, prefix);
    prefix_cache
        .get(&server_id)
        .map(|entry| entry.value().clone())
}

pub async fn set_prefix(
    ctx: &Context,
    guild_id: Option<GuildId>,
    prefix: &str,
    update: bool,
) -> CommandResult {
    let server_id = guild_id.ok_or(WrongGuild)?;

    let mut conn = get_database_conn!(ctx, Result);

    let req = if update {
        println!("Updating prefix to \"{}\"", prefix);
        format!(
            "UPDATE {} SET prefix = :prefix WHERE server_id = :server_id",
            TABLE_PREFIX
        )
    } else {
        println!("Initializing prefix to \"{}\"", prefix);
        format!(
            "INSERT INTO {} (server_id, prefix) VALUES (:server_id, :prefix)",
            TABLE_PREFIX
        )
    };
    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id.0,
            "prefix" => prefix,
        },
    )
    .await?;
    println!("Done.");

    if let Some(prefix_cache) = ctx.data.read().await.get::<PrefixCache>() {
        prefix_cache.insert(server_id, prefix.to_string());
    } else {
        unreachable!();
    }

    Ok(())
}

pub async fn get_minecraft_ip(ctx: &Context, guild_id: Option<GuildId>) -> Option<String> {
    let server_id: u64 = guild_id?.0;

    let mut conn = get_database_conn!(ctx);

    conn.query_first(format!(
        "SELECT mc_ip FROM {} WHERE server_id={}",
        TABLE_MC_SERVER_IP, server_id
    ))
    .await
    .ok()?
}

pub async fn set_minecraft_ip(
    ctx: &Context,
    guild_id: Option<GuildId>,
    ip: &str,
    update: bool,
) -> CommandResult {
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

    let mut conn = get_database_conn!(ctx, Result);

    let req = if update {
        println!("Updating IP to {}", ip);
        format!(
            "UPDATE {} SET mc_ip = :mc_ip WHERE server_id = :server_id",
            TABLE_MC_SERVER_IP
        )
    } else {
        println!("Setting up ip to {}", ip);
        format!(
            "INSERT INTO {} (server_id, mc_ip) VALUES (:server_id, :mc_ip)",
            TABLE_MC_SERVER_IP
        )
    };

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "mc_ip" => ip,
        },
    )
    .await?;
    println!("Done");

    drop(conn);

    Ok(())
}

pub async fn delete_minecraft_ip(ctx: &Context, guild_id: Option<GuildId>) -> CommandResult {
    let server_id: u64 = guild_id.ok_or(WrongGuild)?.0;

    let mut conn = get_database_conn!(ctx, Result);

    let req = format!(
        "DELETE FROM {} WHERE server_id = :server_id LIMIT 1",
        TABLE_MC_SERVER_IP
    );

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id
        },
    )
    .await?;

    Ok(())
}
