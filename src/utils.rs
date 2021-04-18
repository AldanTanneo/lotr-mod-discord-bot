use serde_json::{Error, Value};
use serenity::client::Context;
use serenity::model::prelude::*;

use crate::constants::MAX_JSON_FILE_SIZE;
use JsonMessageError::*;

#[derive(Debug)]
pub enum JsonMessageError {
    FileTooBig,
    DownloadError,
    JsonError(Error),
}

pub async fn get_json_from_message(msg: &Message) -> Result<Value, JsonMessageError> {
    if msg.attachments.is_empty() {
        let content = &msg.content;
        let (a, b) = (
            content.find('{').unwrap_or(0),
            content.rfind('}').unwrap_or(0),
        );
        serde_json::from_str(&content[a..=b]).map_err(JsonError)
    } else {
        let a = &msg.attachments[0];
        if a.size <= MAX_JSON_FILE_SIZE {
            if let Ok(json_data) = a.download().await {
                serde_json::from_slice(&json_data).map_err(JsonError)
            } else {
                Err(DownloadError)
            }
        } else {
            Err(FileTooBig)
        }
    }
}

#[macro_export]
macro_rules! handle_json_error {
    ($ctx:ident, $msg:ident, $error:ident) => {
        match $error {
            FileTooBig => {
                failure!(
                    $ctx,
                    $msg,
                    "Attachment is too big! Filesize must be under {}KB.",
                    $crate::constants::MAX_JSON_FILE_SIZE / 1024
                );
            }
            DownloadError => {
                failure!($ctx, $msg, "Could not download attachment!");
            }
            JsonError(e) => {
                println!("Error reading JSON content: {}", e);
                failure!(
                    $ctx,
                    $msg,
                    "Could not read your JSON content! Check for syntax errors."
                );
            }
        }
    };
}

#[macro_export]
macro_rules! success {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('✅')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {
        $msg.reply($ctx, $single_message).await?;
        success!($ctx, $msg);
    };
    ($ctx:ident, $msg:ident, $($success:tt)*) => {
        $msg.reply($ctx, format!($($success)*)).await?;
        success!($ctx, $msg);
    };
}

#[macro_export]
macro_rules! failure {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('❌')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {
        $msg.reply($ctx, $single_message).await?;
        failure!($ctx, $msg);
    };
    ($ctx:ident, $msg:ident, $($error:tt)*) => {
        $msg.reply($ctx, format!($($error)*)).await?;
        failure!($ctx, $msg);
    };
}

#[macro_export]
macro_rules! is_admin {
    ($ctx:ident, $msg:ident) => {
        is_admin!($ctx, $msg.guild_id, $msg.author.id)
    };
    ($ctx:ident, $guild_id:expr, $user:expr) => {
        $crate::database::admin_data::is_admin_function($ctx, $guild_id, $user)
            .await
            .unwrap_or(false)
    };
}

/// Checks a [`User`]'s permissions.
///
/// Returns `true` if `user` has the [permissions][Permissions] `perm` in the
/// `guild`. If `guild` is [`None`], or if the user lacks the permissions,
/// returns `false`.
pub async fn has_permission(
    ctx: &Context,
    guild: Option<GuildId>,
    user_id: UserId,
    perm: Permissions,
) -> bool {
    if let Some(guild) = guild {
        if let Ok(g) = guild.to_partial_guild(&ctx).await {
            if let Ok(m) = g.member(ctx, user_id).await {
                return m
                    .permissions(ctx)
                    .await
                    .unwrap_or_default()
                    .intersects(perm);
            }
        }
    }
    false
}
