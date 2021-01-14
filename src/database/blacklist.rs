use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{
    id::{ChannelId, GuildId, UserId},
    misc::Mentionable,
    prelude::Message,
};

use super::DatabasePool;
use super::{Blacklist, Blacklist::*};
use crate::constants::{TABLE_CHANNEL_BLACKLIST, TABLE_USER_BLACKLIST};

pub async fn check_blacklist(ctx: &Context, msg: &Message, get_list: bool) -> Option<Blacklist> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
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
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not retrieve the database pool");
            return Ok(());
        }
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
