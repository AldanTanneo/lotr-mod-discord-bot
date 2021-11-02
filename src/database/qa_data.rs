use mysql_async::params;
use mysql_async::prelude::Queryable;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::get_database_conn;

pub async fn get_answer_channel(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_first(
        "SELECT channel_id FROM qa__channels WHERE guild_id = :guild_id AND channel_type = 'answers'", 
        params! {
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
    .map(ChannelId)
}

pub async fn get_question_channel(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_first(
        "SELECT channel_id FROM qa__channels WHERE guild_id = :guild_id AND channel_type = 'questions'", 
        params! {
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
    .map(ChannelId)
}

pub async fn is_qa_moderator(ctx: &Context, user_id: UserId, guild_id: GuildId) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_first(
        "SELECT EXISTS(SELECT perm_id FROM qa__moderators WHERE user_id = :user_id AND guild_id = :guild_id)", 
        params! {
            "user_id" => user_id.0,
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
}

pub async fn is_questions_channel(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_first(
        "SELECT EXISTS(SELECT uid FROM qa__channels WHERE channel_id = :channel_id AND guild_id = :guild_id AND channel_type = 'questions')", 
        params! {
            "channel_id" => channel_id.0,
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
}

pub async fn add_qa_moderator(ctx: &Context, user_id: UserId, guild_id: GuildId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        "INSERT INTO qa__moderators (guild_id, user_id) VALUES (:guild_id, :user_id)",
        params! {
            "guild_id" => guild_id.0,
            "user_id" => user_id.0
        },
    )
    .await?;

    Ok(())
}

pub async fn remove_qa_moderator(
    ctx: &Context,
    user_id: UserId,
    guild_id: GuildId,
) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        "DELETE FROM qa__moderators WHERE user_id = :user_id AND guild_id = :guild_id LIMIT 1",
        params! {
            "user_id" => user_id.0,
            "guild_id" => guild_id.0
        },
    )
    .await?;

    Ok(())
}

pub async fn set_answer_channel(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    let query = if get_answer_channel(ctx, guild_id).await.is_some() {
        "UPDATE qa__channels SET channel_id = :channel_id WHERE guild_id = :guild_id AND channel_type = 'answers'"
    } else {
        "INSERT INTO qa__channels (guild_id, channel_id, channel_type) VALUES (:guild_id, :channel_id, 'answers')"
    };

    conn.exec_drop(
        query,
        params! {
            "guild_id" => guild_id.0,
            "channel_id" => channel_id.0,
        },
    )
    .await?;

    Ok(())
}

pub async fn set_question_channel(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    let query = if get_question_channel(ctx, guild_id).await.is_some() {
        "UPDATE qa__channels SET channel_id = :channel_id WHERE guild_id = :guild_id AND channel_type = 'questions'"
    } else {
        "INSERT INTO qa__channels (guild_id, channel_id, channel_type) VALUES (:guild_id, :channel_id, 'questions')"
    };

    conn.exec_drop(
        query,
        params! {
            "guild_id" => guild_id.0,
            "channel_id" => channel_id.0,
        },
    )
    .await?;

    Ok(())
}

pub async fn disable_qa(ctx: &Context, guild_id: GuildId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        "DELETE FROM qa__channels WHERE guild_id = :guild_id",
        params! {
            "guild_id" => guild_id.0
        },
    )
    .await?;

    Ok(())
}
