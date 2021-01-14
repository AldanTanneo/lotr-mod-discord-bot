use mysql_async::prelude::*;
use rand::seq::IteratorRandom;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::{GuildId, UserId};

use super::DatabasePool;
use crate::constants::{OWNER_ID, TABLE_ADMINS, TABLE_FLOPPA};

fn choose_from_ids(vec: Vec<u32>) -> u32 {
    let mut rng = rand::thread_rng();
    let id = vec.iter().choose(&mut rng).unwrap_or(&1);
    *id
}

pub async fn get_floppa(ctx: &Context, n: Option<u32>) -> Option<String> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let ids: Vec<u32> = conn
        .exec_map(
            format!("SELECT id FROM {}", TABLE_FLOPPA).as_str(),
            (),
            |id| id,
        )
        .await
        .ok()?;

    if ids.is_empty() {
        return None;
    }

    let floppa_id = if let Some(n) = n {
        if ids.contains(&n) {
            n
        } else {
            *ids.get(0.max((n - 1) as usize % ids.len()))?
        }
    } else {
        choose_from_ids(ids)
    };

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
    let mut conn = pool.get_conn().await?;

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
            .dm(ctx, |m| {
                m.content("Tried to add floppa that already exists!")
            })
            .await?;
    }

    drop(conn);

    Ok(())
}

pub async fn is_floppadmin(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
) -> Option<bool> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;
    let server_id: u64 = guild_id?.0;

    let res = conn
        .exec_map(
            format!(
                "SELECT user_id FROM {} WHERE server_id=:server_id AND floppadmin = true",
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

    Some(res.contains(&user_id))
}
