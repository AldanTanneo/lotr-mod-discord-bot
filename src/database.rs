use mysql_async::prelude::*;
use mysql_async::*;
use serenity::client::Context;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";

#[derive(Debug, PartialEq, Eq)]
struct ServerPrefix {
    server_id: u64,
    prefix: Option<String>,
}

struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> String {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = Arc::try_unwrap(pool)
        .unwrap()
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
        set_prefix(ctx, guild_id, "!".to_string()).await.unwrap();
        "!".to_string()
    }
}

pub async fn set_prefix(ctx: &Context, guild_id: Option<GuildId>, prefix: String) -> Result<()> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = Arc::try_unwrap(pool)
        .unwrap()
        .get_conn()
        .await
        .expect("Could not connect to database");
    let server_id: u64 = if let Some(id) = guild_id {
        id.into()
    } else {
        0
    };
    /* conn.exec_first(
        format!(
            "INSERT INTO {} (server_id, prefix) VALUES (:server_id, :prefix)",
            TABLE_PREFIX
        ),
        params! {
            "server_id" => server_id,
            "prefix" => Some(prefix),
        },
    ); */
    conn.exec_batch(
        r"INSERT INTO payment (customer_id, amount, account_name)
      VALUES (:customer_id, :amount, :account_name)",
        vec![ServerPrefix {
            server_id: server_id,
            prefix: Some(prefix),
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
