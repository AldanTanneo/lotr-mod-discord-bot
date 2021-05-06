use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::model::prelude::*;

use super::DatabasePool;
use crate::constants::TABLE_LIST_GUILDS;

pub async fn update_list_guilds(ctx: &Context) -> Result<i64, CommandError> {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not get database pool");
            return Ok(i64::MIN);
        }
    };
    let mut conn = pool.get_conn().await?;

    let before: i64 = dbg!(conn
        .query_first(format!("SELECT COUNT(guild_id) FROM {}", TABLE_LIST_GUILDS))
        .await?
        .unwrap_or(0));

    conn.query_drop(format!(
        "DELETE FROM {} WHERE guild_name != 'last_cleanup'",
        TABLE_LIST_GUILDS
    ))
    .await?;

    let mut id = GuildId(0);
    while let Ok(vec) = ctx
        .http
        .get_guilds(&serenity::http::GuildPagination::After(id), 100)
        .await
    {
        if vec.is_empty() {
            break;
        }
        id = vec[vec.len() - 1].id;
        conn.exec_batch(
            format!(
                "INSERT INTO {} (guild_id, guild_name) VALUES (:guild_id, :guild_name)",
                TABLE_LIST_GUILDS
            ),
            vec.iter().map(|guild| {
                params! {
                    "guild_id" => guild.id.0,
                    "guild_name" => &guild.name
                }
            }),
        )
        .await?;
    }

    let after: i64 = dbg!(conn
        .query_first(format!("SELECT COUNT(guild_id) FROM {}", TABLE_LIST_GUILDS))
        .await?
        .unwrap_or(0));

    Ok(after - before)
}

/*
pub async fn cleanup_database(ctx: &Context) -> Result<i64, CommandError> {
    let res = update_list_guilds(ctx).await;
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not get database pool");
            return Ok(u64::MAX);
        }
    };
    let mut conn = pool.get_conn().await?;

    conn.query_drop(format!("DELETE FROM {} WHERE NOT EXISTS(SELECT"))
    res
}
*/
