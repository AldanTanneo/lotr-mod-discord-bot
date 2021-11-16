use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;

use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID, TABLE_CHANNEL_BLACKLIST, TABLE_USER_BLACKLIST};
use crate::utils::NotInGuild;
use crate::{get_database_conn, is_admin};

pub async fn check_blacklist(
    ctx: &Context,
    server_id: GuildId,
    user_id: UserId,
    channel_id: ChannelId,
) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    let user_blacklist: bool = conn
        .query_first(format!(
            "SELECT EXISTS(SELECT id FROM {} WHERE server_id={} AND user_id={} LIMIT 1)",
            TABLE_USER_BLACKLIST, server_id.0, user_id.0
        ))
        .await
        .ok()??;

    let channel_blacklist: bool = conn
        .query_first(format!(
            "SELECT EXISTS(SELECT id FROM {} WHERE server_id={} AND channel_id={} LIMIT 1)",
            TABLE_CHANNEL_BLACKLIST, server_id.0, channel_id.0
        ))
        .await
        .ok()??;

    Some(channel_blacklist || user_blacklist)
}

pub async fn get_blacklist(
    ctx: &Context,
    server_id: GuildId,
) -> Option<(Vec<UserId>, Vec<ChannelId>)> {
    let mut conn = get_database_conn!(ctx);

    let user_blacklist: Vec<UserId> = conn
        .exec_map(
            format!(
                "SELECT user_id as id FROM {} WHERE server_id=:server_id",
                TABLE_USER_BLACKLIST
            )
            .as_str(),
            params! {
                "server_id" => server_id.0,
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
                "server_id" => server_id.0,
            },
            ChannelId,
        )
        .await
        .ok()?;

    Some((user_blacklist, channel_blacklist))
}

pub async fn update_blacklist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let pguild = server_id.to_partial_guild(ctx).await?;

    for user in &msg.mentions {
        if let Ok(member) = pguild.member(ctx, user.id).await {
            if user.id == OWNER_ID
                || is_admin!(ctx, server_id, user.id)
                || member
                    .permissions(ctx)
                    .unwrap_or_default()
                    .intersects(MANAGE_BOT_PERMS)
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
        if check_blacklist(ctx, server_id, user.id, ChannelId(0)).await == Some(true) {
            conn.exec_drop(
                format!(
                    "DELETE FROM {} WHERE server_id = :server_id AND user_id = :user_id LIMIT 1",
                    TABLE_USER_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id.0,
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
                    "server_id" => server_id.0,
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
        if check_blacklist(ctx, server_id, UserId(0), channel).await == Some(true) {
            conn.exec_drop(
                format!(
                    "DELETE FROM {} WHERE server_id = :server_id AND channel_id = :channel_id LIMIT 1",
                    TABLE_CHANNEL_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id.0,
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
            conn.exec_drop(
                format!(
                    "INSERT INTO {} (server_id, channel_id) VALUES (:server_id, :channel_id)",
                    TABLE_CHANNEL_BLACKLIST
                )
                .as_str(),
                params! {
                    "server_id" => server_id.0,
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
