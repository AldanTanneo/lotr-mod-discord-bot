use mysql_async::prelude::*;
use mysql_async::*;
use rand::seq::IteratorRandom;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::{GuildId, UserId};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

const OWNER_ID: UserId = UserId(405421991777009678);
const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";
const TABLE_ADMINS: &str = "bot_admins";
const TABLE_FLOPPA: &str = "floppa_images";

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

    drop(conn);

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
) -> CommandResult {
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
    conn.exec_drop(
        req.as_str(),
        params! {
            "server_id" => server_id,
            "prefix" => prefix,
        },
    )
    .await?;

    drop(conn);

    Ok(())
}

pub async fn get_admins(ctx: &Context, guild_id: Option<GuildId>) -> Option<Vec<UserId>> {
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
        .exec_map(
            format!(
                "SELECT user_id FROM {} WHERE server_id={}",
                TABLE_ADMINS, server_id
            )
            .as_str(),
            (),
            UserId,
        )
        .await
        .ok()?;

    drop(conn);

    Some(res)
}

pub async fn add_admin(ctx: &Context, guild_id: Option<GuildId>, user_id: UserId) -> CommandResult {
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
    let server_id: u64 = if let Some(id) = guild_id { id.0 } else { 0 };

    conn.exec_drop(
        format!(
            "INSERT INTO {} (server_id, user_id) VALUES (:server_id, :user_id)",
            TABLE_ADMINS
        )
        .as_str(),
        params! {
            "server_id" => server_id,
            "user_id" => user_id.as_u64(),
        },
    )
    .await?;

    drop(conn);

    Ok(())
}

pub async fn remove_admin(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
) -> CommandResult {
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
        *id.as_u64()
    } else {
        0
    };

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id",
            TABLE_ADMINS
        )
        .as_str(),
        params! {
            "server_id" => server_id,
            "user_id" => user_id.as_u64(),
        },
    )
    .await?;

    Ok(())
}

fn choose_from_ids(vec: Vec<u32>) -> u32 {
    let mut rng = rand::thread_rng();
    let id = vec.iter().choose(&mut rng).unwrap_or(&1);
    *id
}

pub async fn get_floppa(ctx: &Context) -> Option<String> {
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

    let ids: Vec<u32> = conn
        .exec_map(
            format!("SELECT id FROM {}", TABLE_FLOPPA).as_str(),
            (),
            |id| id,
        )
        .await
        .ok()?;

    let floppa_id = choose_from_ids(ids);

    let res = conn
        .query_first(format!(
            "SELECT image_url FROM {} WHERE id={}",
            TABLE_FLOPPA, floppa_id
        ))
        .await;

    drop(conn);

    if let Ok(url) = res {
        url
    } else {
        None
    }
}

pub async fn add_floppa(ctx: &Context, floppa_url: String) -> CommandResult {
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

    let images: Vec<String> = conn
        .exec_map(
            format!("SELECT image_url FROM {}", TABLE_FLOPPA).as_str(),
            (),
            |url| url,
        )
        .await?;

    println!("Retrieved floppa urls");

    if !images.contains(&floppa_url) {
        conn.exec_drop(
            format!(
                "INSERT INTO {} (image_url) VALUES (:image_url)",
                TABLE_FLOPPA
            )
            .as_str(),
            params! {
                "image_url" => floppa_url,
            },
        )
        .await?;
        println!("Successfully executed query!");
    } else {
        OWNER_ID
            .to_user(ctx)
            .await?
            .dm(ctx, |m| m.content("Floppa already exists!"))
            .await?;
    }

    drop(conn);

    Ok(())
}
