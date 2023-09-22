use dashmap::DashMap;
use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use crate::constants::{TABLE_MC_SERVER_IP, TABLE_PREFIX};
use crate::get_database_conn;

#[derive(Debug, Clone)]
pub struct PrefixCache(Arc<DashMap<GuildId, String>>);

impl TypeMapKey for PrefixCache {
    type Value = Self;
}

impl std::ops::Deref for PrefixCache {
    type Target = DashMap<GuildId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for PrefixCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixCache {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

pub async fn get_prefix(ctx: &Context, server_id: GuildId) -> Option<String> {
    if server_id == GuildId(0) {
        return Some(String::from("!"));
    }

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
        println!("Initializing prefix for {server_id:?}");
        set_prefix(ctx, server_id, "!").await.ok()?;
        println!("Prefix initialized successfully");
        "!".to_string()
    };

    prefix_cache.insert(server_id, prefix);
    prefix_cache
        .get(&server_id)
        .map(|entry| entry.value().clone())
}

pub async fn set_prefix(ctx: &Context, server_id: GuildId, prefix: &str) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    println!("Setting prefix to {prefix} in {server_id}");
    conn.exec_drop(
        format!(
            "REPLACE INTO {TABLE_PREFIX} (server_id, prefix) VALUES (:server_id, :prefix)"
        ),
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

pub async fn get_minecraft_ip(ctx: &Context, server_id: GuildId) -> Option<String> {
    let mut conn = get_database_conn!(ctx);

    conn.query_first(format!(
        "SELECT mc_ip FROM {} WHERE server_id={}",
        TABLE_MC_SERVER_IP, server_id.0
    ))
    .await
    .ok()?
}

pub async fn set_minecraft_ip(ctx: &Context, server_id: GuildId, ip: &str) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    println!("Setting up ip to {ip}");

    conn.exec_drop(
        format!(
            "REPLACE INTO {TABLE_MC_SERVER_IP} (server_id, mc_ip) VALUES (:server_id, :mc_ip)"
        ),
        params! {
            "server_id" => server_id.0,
            "mc_ip" => ip,
        },
    )
    .await?;
    println!("Done");

    drop(conn);

    Ok(())
}

pub async fn delete_minecraft_ip(ctx: &Context, server_id: GuildId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    let req = format!(
        "DELETE FROM {TABLE_MC_SERVER_IP} WHERE server_id = :server_id LIMIT 1"
    );

    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id.0
        },
    )
    .await?;

    Ok(())
}
