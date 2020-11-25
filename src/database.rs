use mysql_async::prelude::*;
use mysql_async::*;
use serenity::client::Context;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}

#[derive(Debug, PartialEq, Eq)]
struct ServerPrefix {
    server_id: u64,
    prefix: Option<String>,
}

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> String {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool
        .get_conn()
        .await
        .expect("Could not connect to database");
    let server_id: u64 = if let Some(id) = guild_id {
        id.into()
    } else {
        0
    };
    let res = conn
        .query_first(format!(
            "SELECT prefix FROM {} WHERE server_id={}",
            TABLE_PREFIX, server_id
        ))
        .await;
    if let Ok(Some(prefix)) = res {
        prefix
    } else {
        set_prefix(ctx, guild_id, "!", false).await.unwrap();
        "!".to_string()
    }
}

pub async fn set_prefix(
    ctx: &Context,
    guild_id: Option<GuildId>,
    prefix: &str,
    update: bool,
) -> Result<()> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool
        .get_conn()
        .await
        .expect("Could not connect to database");
    let server_id: u64 = if let Some(id) = guild_id {
        id.into()
    } else {
        0
    };
    let req = if update {
        format!(
            "UPDATE {} SET prefix = :prefix WHERE server_id = :server_id",
            TABLE_PREFIX
        )
    } else {
        format!(
            "INSERT INTO {} (server_id, prefix) VALUES (:server_id, :prefix)",
            TABLE_PREFIX
        )
    };
    conn.exec_batch(
        req.as_str(),
        vec![ServerPrefix {
            server_id,
            prefix: Some(prefix.to_string()),
        }]
        .iter()
        .map(|p| {
            params! {
                "server_id" => p.server_id,
                "prefix" => &p.prefix,
            }
        }),
    )
    .await?;
    Ok(())
}
