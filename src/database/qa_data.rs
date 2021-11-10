use dashmap::DashMap;
use mysql_async::params;
use mysql_async::prelude::Queryable;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;

use crate::get_database_conn;

#[derive(Debug, Clone, Default)]
pub struct QaChannels {
    pub answers: Option<ChannelId>,
    pub questions: Option<ChannelId>,
}

#[derive(Debug, Clone)]
pub struct QaChannelsCache(Arc<DashMap<GuildId, QaChannels>>);

impl TypeMapKey for QaChannelsCache {
    type Value = Self;
}

impl std::ops::Deref for QaChannelsCache {
    type Target = DashMap<GuildId, QaChannels>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for QaChannelsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl QaChannelsCache {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

async fn populate_cache(ctx: &Context, guild_id: GuildId) -> Option<QaChannels> {
    let mut conn = get_database_conn!(ctx);

    println!("Setting up cache...");

    let answers = conn.exec_first(
        "SELECT channel_id FROM qa__channels WHERE guild_id = :guild_id AND channel_type = 'answers'", 
        params! {
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
    .map(ChannelId);

    let questions = conn.exec_first(
        "SELECT channel_id FROM qa__channels WHERE guild_id = :guild_id AND channel_type = 'questions'", 
        params! {
            "guild_id" => guild_id.0
    })
    .await
    .ok()?
    .map(ChannelId);

    let channels = ctx
        .data
        .read()
        .await
        .get::<QaChannelsCache>()?
        .clone()
        .entry(guild_id)
        .or_insert(QaChannels { answers, questions })
        .value()
        .clone();

    println!("Cache successfully setup");

    Some(channels)
}

pub async fn get_answer_channel(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    let channels_cache = ctx.data.read().await.get::<QaChannelsCache>()?.clone();

    let qa_channels = channels_cache.get(&guild_id);

    if let Some(qa_channels) = qa_channels {
        qa_channels.answers
    } else {
        populate_cache(ctx, guild_id).await?.answers
    }
}

pub async fn get_question_channel(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    let channels_cache = ctx.data.read().await.get::<QaChannelsCache>()?.clone();

    let qa_channels = channels_cache.get(&guild_id);

    if let Some(qa_channels) = qa_channels {
        qa_channels.questions
    } else {
        populate_cache(ctx, guild_id).await?.questions
    }
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

pub async fn get_qa_moderator_list(ctx: &Context, guild_id: GuildId) -> Option<Vec<UserId>> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_map(
        "SELECT user_id FROM qa__moderators WHERE guild_id = :guild_id",
        params! {
            "guild_id" => guild_id.0
        },
        UserId,
    )
    .await
    .ok()
}

pub async fn is_questions_channel(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<bool> {
    get_question_channel(ctx, guild_id)
        .await
        .map(|c| c == channel_id)
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

    let channels_cache = ctx
        .data
        .read()
        .await
        .get::<QaChannelsCache>()
        .expect("There should be a q&a channel cache in the Typemap")
        .clone();

    {
        let mut cache_entry = channels_cache.entry(guild_id).or_default();

        cache_entry.answers = Some(channel_id);
    }

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

    let channels_cache = ctx
        .data
        .read()
        .await
        .get::<QaChannelsCache>()
        .expect("There should be a q&a channel cache in the Typemap")
        .clone();

    {
        let mut cache_entry = channels_cache.entry(guild_id).or_default();

        cache_entry.questions = Some(channel_id);
    }

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

    let channels_cache = ctx
        .data
        .read()
        .await
        .get::<QaChannelsCache>()
        .expect("There should be a q&a channel cache in the Typemap")
        .clone();

    {
        let mut cache_entry = channels_cache.entry(guild_id).or_default();

        *cache_entry = QaChannels::default();
    }

    Ok(())
}
