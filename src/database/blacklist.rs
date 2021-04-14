use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{
    error::Error::WrongGuild,
    id::{ChannelId, GuildId, UserId},
    misc::Mentionable,
    prelude::Message,
};

use super::admin_data::is_admin;
use super::{Blacklist, Blacklist::*, DatabasePool};
use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID, TABLE_CHANNEL_BLACKLIST, TABLE_USER_BLACKLIST};

pub async fn check_blacklist(ctx: &Context, msg: &Message, get_list: bool) -> Option<Blacklist> {
    let server_id: u64 = msg.guild_id?.0;

    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    if get_list {
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

        Some(List(user_blacklist, channel_blacklist))
    } else {
        let user_blacklist: bool = conn
            .query_first(format!(
                "SELECT EXISTS(SELECT id FROM {} WHERE server_id={} AND user_id={} LIMIT 1)",
                TABLE_USER_BLACKLIST, server_id, msg.author.id.0
            ))
            .await
            .ok()??;

        let channel_blacklist: bool = conn
            .query_first(format!(
                "SELECT EXISTS(SELECT id FROM {} WHERE server_id={} AND channel_id={} LIMIT 1)",
                TABLE_CHANNEL_BLACKLIST, server_id, msg.channel_id.0
            ))
            .await
            .ok()??;

        Some(IsBlacklisted(channel_blacklist || user_blacklist))
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

    let server_id: u64 = msg.guild_id.ok_or(WrongGuild)?.0;
    let pguild = GuildId(server_id).to_partial_guild(ctx).await?;
    let (users, channels) = check_blacklist(ctx, msg, true)
        .await
        .unwrap_or(IsBlacklisted(true))
        .get_list();

    for user in &msg.mentions {
        println!("user...");
        if let Ok(member) = pguild.member(ctx, user.id).await {
            if user.id == OWNER_ID
                || is_admin(ctx, msg.guild_id, user.id).await.is_some()
                || member
                    .permissions(ctx)
                    .await
                    .unwrap_or_default()
                    .intersects(*MANAGE_BOT_PERMS)
            {
                msg.channel_id
                    .say(
                        ctx,
                        format!("You cannot add {} to the blacklist!", user.name),
                    )
                    .await?;
                continue;
            }
        }
        if users.contains(&user.id) {
            println!("deleting {}", user.id);
            conn.exec_drop(
                format!(
                    "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id LIMIT 1",
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
                    "DELETE FROM {} WHERE server_id = :server_id AND channel_id = :channel_id LIMIT 1",
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
