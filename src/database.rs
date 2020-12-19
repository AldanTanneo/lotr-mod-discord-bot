use mysql_async::prelude::*;
use mysql_async::*;
use rand::seq::IteratorRandom;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{
    id::{ChannelId, GuildId, UserId},
    misc::Mentionable,
    prelude::Message,
};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use std::cmp;

use Blacklist::*;

const OWNER_ID: UserId = UserId(405421991777009678);
const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";
const TABLE_ADMINS: &str = "bot_admins";
const TABLE_FLOPPA: &str = "floppa_images";
const TABLE_USER_BLACKLIST: &str = "user_blacklist";
const TABLE_CHANNEL_BLACKLIST: &str = "channel_blacklist";

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ServerPrefix {
    pub(crate) server_id: u64,
    pub(crate) prefix: Option<String>,
}

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> Option<String> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let server_id: u64 = guild_id?.0;

    let res = conn
        .query_first(format!(
            "SELECT prefix FROM {} WHERE server_id={}",
            TABLE_PREFIX, server_id
        ))
        .await;

    drop(conn);

    if let Ok(prefix) = res {
        prefix
    } else {
        set_prefix(ctx, guild_id, "!", false).await.ok()?;
        Some("!".to_string())
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
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

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
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;
    let server_id: u64 = guild_id?.0;

    let res = conn
        .exec_map(
            format!(
                "SELECT user_id FROM {} WHERE server_id=:server_id",
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

    Some(res)
}

pub async fn add_admin(
    ctx: &Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
    update: bool,
    floppadmin: bool,
) -> CommandResult {
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
    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

    if update {
        // UPDATE {} SET prefix = :prefix WHERE server_id = :server_id
        conn.exec_drop(
            format!(
                "UPDATE {} SET floppadmin = :floppa WHERE server_id = :server_id AND user_id = :user_id",
                TABLE_ADMINS
            )
            .as_str(),
            params! {
                "server_id" => server_id,
                "user_id" => user_id.0,
                "floppa" => floppadmin,
            },
        )
        .await?;
    } else {
        conn.exec_drop(
            format!(
                "INSERT INTO {} (server_id, user_id, floppadmin) VALUES (:server_id, :user_id, :floppa)",
                TABLE_ADMINS
            )
            .as_str(),
            params! {
                "server_id" => server_id,
                "user_id" => user_id.0,
                "floppa" => floppadmin,
            },
        )
        .await?;
    }

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
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = guild_id.unwrap_or(GuildId(0)).0;

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id",
            TABLE_ADMINS
        )
        .as_str(),
        params! {
            "server_id" => server_id,
            "user_id" => user_id.0,
        },
    )
    .await?;

    drop(conn);

    Ok(())
}

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
    
    
    let floppa_id = if n.is_some() && ids.len() > 0 { 
        *ids.get(cmp::max(0, n.unwrap() as usize % ids.len())).unwrap()
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
            .dm(ctx, |m| m.content("Floppa already exists!"))
            .await?;
    }

    drop(conn);

    Ok(())
}

pub enum Blacklist {
    IsBlacklisted(bool),
    List(Vec<UserId>, Vec<ChannelId>),
}

impl Blacklist {
    pub fn is_blacklisted(&self) -> bool {
        match self {
            IsBlacklisted(b) => *b,
            _ => false,
        }
    }

    pub fn get_list(&self) -> (Vec<UserId>, Vec<ChannelId>) {
        match self {
            List(a, b) => (a.to_vec(), b.to_vec()),
            _ => (vec![], vec![]),
        }
    }
}

pub async fn check_blacklist(ctx: &Context, msg: &Message, get_list: bool) -> Option<Blacklist> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let server_id: u64 = msg.guild_id?.0;

    let user_blacklist: Vec<UserId> = conn
        .exec_map(
            format!(
                "SELECT user_id as id FROM {} WHERE server_id=:server_id",
                TABLE_USER_BLACKLIST
            )
            .as_str(),
            params! {
                "server_id" => server_id,
            },
            UserId,
        )
        .await
        .ok()?;

    let channel_blacklist: Vec<ChannelId> = conn
        .exec_map(
            format!(
                "SELECT channel_id as id FROM {} WHERE server_id=:server_id",
                TABLE_CHANNEL_BLACKLIST
            )
            .as_str(),
            params! {
                "server_id" => server_id,
            },
            ChannelId,
        )
        .await
        .ok()?;

    if get_list {
        Some(List(user_blacklist, channel_blacklist))
    } else {
        let (user_id, channel_id) = (msg.author.id, msg.channel_id);
        Some(IsBlacklisted(
            channel_blacklist.contains(&channel_id) || user_blacklist.contains(&user_id),
        ))
    }
}

pub async fn update_blacklist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("updating blacklist");
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool.get_conn().await?;

    let server_id: u64 = msg.guild_id.unwrap_or(GuildId(0)).0;
    let (users, channels) = check_blacklist(ctx, msg, true)
        .await
        .unwrap_or(IsBlacklisted(true))
        .get_list();

    for user in &msg.mentions {
        println!("user...");
        if users.contains(&user.id) {
            println!("deleting");
            conn.exec_drop(
                format!(
                    "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id",
                    TABLE_USER_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id,
                    "user_id" => user.id.0,
                },
            )
            .await?;
            msg.channel_id
                .say(
                    ctx,
                    format!("Removed user {} from the blacklist", user.name),
                )
                .await?;
        } else {
            println!("adding");
            conn.exec_drop(
                format!(
                    "INSERT INTO {} (server_id, user_id) VALUES (:server_id, :user_id)",
                    TABLE_USER_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id,
                    "user_id" => user.id.0,
                },
            )
            .await?;
            msg.channel_id
                .say(ctx, format!("Added user {} to the blacklist", user.name))
                .await?;
        }
    }

    let mentioned_channels = args
        .trimmed()
        .iter()
        .map(|a| serenity::utils::parse_channel(a.unwrap_or_else(|_| "".to_string())))
        .filter(|c| c.is_some())
        .map(|c| ChannelId(c.unwrap()));

    for channel in mentioned_channels {
        println!("channel...");
        if channels.contains(&channel) {
            println!("deleting");
            conn.exec_drop(
                format!(
                    "DELETE FROM {} WHERE server_id = :server_id AND channel_id = :channel_id",
                    TABLE_CHANNEL_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id,
                    "channel_id" => channel.0,
                },
            )
            .await?;
            msg.channel_id
                .say(
                    ctx,
                    format!("Removed channel {} from the blacklist", channel.mention()),
                )
                .await?;
        } else {
            println!("adding");
            conn.exec_drop(
                format!(
                    "INSERT INTO {} (server_id, channel_id) VALUES (:server_id, :channel_id)",
                    TABLE_CHANNEL_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id,
                    "channel_id" => channel.0,
                },
            )
            .await?;
            msg.channel_id
                .say(
                    ctx,
                    format!("Added channel {} to the blacklist", channel.mention()),
                )
                .await?;
        }
    }

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
