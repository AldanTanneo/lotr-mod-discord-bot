use serde::de::DeserializeOwned;
use serde_json::Error;
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

pub async fn get_json_from_message<T: DeserializeOwned>(
    msg: &Message,
) -> Result<T, JsonMessageError> {
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
            $crate::utils::JsonMessageError::FileTooBig => {
                $crate::failure!(
                    $ctx,
                    $msg,
                    "Attachment is too big! Filesize must be under {}KB.",
                    $crate::constants::MAX_JSON_FILE_SIZE / 1024
                );
            }
            $crate::utils::JsonMessageError::DownloadError => {
                $crate::failure!($ctx, $msg, "Could not download attachment!");
            }
            $crate::utils::JsonMessageError::JsonError(e) => {
                println!("Error reading JSON content: {}", e);
                $crate::failure!(
                    $ctx,
                    $msg,
                    "Could not read your JSON content! Check for syntax errors."
                );
            }
        }
    };
}

#[macro_export]
macro_rules! get_reqwest_client {
    ($ctx:ident) => {{
        let data_read = $ctx.data.read().await;
        data_read.get::<$crate::api::ReqwestClient>()?.clone()
    }};
    ($ctx:ident, Result) => {{
        let data_read = $ctx.data.read().await;
        if let Some(rclient) = data_read.get::<$crate::api::ReqwestClient>() {
            rclient.clone()
        } else {
            println!("Could not get reqwest client");
            return Ok(());
        }
    }};
}

#[macro_export]
macro_rules! get_database_conn {
    ($ctx:ident) => {{
        let pool = {
            let data_read = $ctx.data.read().await;
            data_read.get::<$crate::database::DatabasePool>()?.clone()
        };
        pool.get_conn().await.ok()?
    }};
    ($ctx:ident, Option) => {
        get_database_conn!($ctx)
    };
    ($ctx:ident, Result) => {
        get_database_conn!($ctx, Result, Ok(()));
    };
    ($ctx:ident, Result, $default:expr) => {{
        let pool = {
            let data_read = $ctx.data.read().await;
            if let Some(p) = data_read.get::<$crate::database::DatabasePool>() {
                p.clone()
            } else {
                println!("Could not get database pool");
                return $default;
            }
        };
        pool.get_conn().await?
    }};
}

#[macro_export]
macro_rules! success {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('✅')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {
        $msg.reply($ctx, $single_message).await?;
        $crate::success!($ctx, $msg);
    };
    ($ctx:ident, $msg:ident, $($success:tt)*) => {
        $msg.reply($ctx, format!($($success)*)).await?;
        $crate::success!($ctx, $msg);
    };
}

#[macro_export]
macro_rules! failure {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('❌')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {
        $msg.reply($ctx, $single_message).await?;
        $crate::failure!($ctx, $msg);
    };
    ($ctx:ident, $msg:ident, $($error:tt)*) => {
        $msg.reply($ctx, format!($($error)*)).await?;
        $crate::failure!($ctx, $msg);
    };
}

#[macro_export]
#[allow(dead_code)]
macro_rules! warn {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('⚠')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {
        $msg.reply($ctx, $single_message).await?;
        $crate::warn!($ctx, $msg);
    };
    ($ctx:ident, $msg:ident, $($error:tt)*) => {
        $msg.reply($ctx, format!($($error)*)).await?;
        $crate::warn!($ctx, $msg);
    };
}

#[macro_export]
macro_rules! is_admin {
    ($ctx:ident, $msg:ident) => {
        $crate::is_admin!($ctx, $msg.guild_id, $msg.author.id)
    };
    ($ctx:ident, $guild_id:expr, $user:expr) => {
        $crate::database::admin_data::is_admin_function($ctx, $guild_id, $user)
            .await
            .unwrap_or_default()
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

pub fn parse_motd<T: ToString>(motd: T) -> String {
    let motd = motd.to_string();
    let mut res = String::with_capacity(motd.len());
    let mut stack: Vec<&str> = Vec::new();
    let mut is_token = false;
    for c in motd.chars() {
        if c == '§' {
            is_token = true;
        } else if is_token {
            is_token = false;
            match c {
                '0'..='9' | 'a'..='f' | 'k' | 'r' => {
                    if !stack.is_empty() {
                        stack.drain(..).rev().for_each(|s| res.push_str(s));
                        res.push('\u{200B}');
                    }
                }
                'l' => {
                    stack.push("**");
                    res.push_str("**");
                }

                'n' => {
                    stack.push("__");
                    res.push_str("__");
                }
                'm' => {
                    stack.push("~~");
                    res.push_str("~~");
                }
                'o' => {
                    stack.push("*");
                    res.push('*');
                }
                _ => {
                    res.push('§');
                    res.push(c)
                }
            }
        } else {
            res.push(c);
        }
    }
    stack.drain(..).rev().for_each(|t| res.push_str(t));
    res
}
