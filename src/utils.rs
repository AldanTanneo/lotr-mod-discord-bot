use serde::de::DeserializeOwned;
use serenity::client::Context;
use serenity::model::prelude::*;

use crate::constants::MAX_JSON_FILE_SIZE;

/// Custom error for unwrapping `msg.guild_id`
#[derive(Debug)]
pub struct NotInGuild;

impl std::fmt::Display for NotInGuild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not in a guild")
    }
}

impl std::error::Error for NotInGuild {}

#[derive(Debug)]
pub enum JsonMessageError {
    FileTooBig(u64),
    DownloadError(serenity::Error),
    JsonError(serde_json::Error),
}

use JsonMessageError::*;

impl std::fmt::Display for JsonMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileTooBig(size) => {
                write!(f, "File too big to download: {}", bytesize::ByteSize(*size))
            }
            DownloadError(e) => write!(f, "Could not download attachment: {}", e),
            JsonError(e) => write!(f, "Error reading JSON content: {}", e),
        }
    }
}

impl std::error::Error for JsonMessageError {}

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
            match a.download().await {
                Ok(json_data) => serde_json::from_slice(&json_data).map_err(JsonError),
                Err(e) => Err(DownloadError(e)),
            }
        } else {
            Err(FileTooBig(a.size))
        }
    }
}

#[macro_export]
macro_rules! handle_json_error {
    ($ctx:ident, $msg:ident, $error:ident) => {
        match $error {
            $crate::utils::JsonMessageError::FileTooBig(size) => {
                $crate::failure!(
                    $ctx,
                    $msg,
                    "Attachment is too big! Filesize must be under {}. Attached file size: {}",
                    bytesize::ByteSize($crate::constants::MAX_JSON_FILE_SIZE),
                    bytesize::ByteSize(size)
                );
            }
            $crate::utils::JsonMessageError::DownloadError(e) => {
                println!(
                    "=== ERROR ===\nCould not download attachment: {}\n=== END ===",
                    e
                );
                $crate::failure!($ctx, $msg, "Could not download attachment!");
            }
            $crate::utils::JsonMessageError::JsonError(e) => {
                $crate::failure!($ctx, $msg, "Error reading JSON content: {}", e);
            }
        }
    };
}

#[macro_export]
macro_rules! get_reqwest_client {
    ($ctx:ident) => {{
        let data_read = $ctx.data.read().await;
        data_read
            .get::<$crate::api::ReqwestClient>()
            .expect("Expected a reqwest client in the type map")
            .clone()
    }};
}

#[macro_export]
macro_rules! get_database_conn {
    ($ctx:ident) => {{
        let pool = {
            let data_read = $ctx.data.read().await;
            data_read
                .get::<$crate::database::DatabasePool>()
                .expect("Expected a database pool in the type map")
                .clone()
        };
        pool.get_conn()
            .await
            .ok()
            .expect("Could not retrieve connection to database")
    }};
}

#[macro_export]
macro_rules! success {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('✅')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {{
        $msg.reply($ctx, $single_message).await?;
        $crate::success!($ctx, $msg);
    }};
    ($ctx:ident, $msg:ident, $($success:tt)*) => {{
        $msg.reply($ctx, format!($($success)*)).await?;
        $crate::success!($ctx, $msg);
    }};
}

#[macro_export]
macro_rules! failure {
    ($ctx:ident, $msg:ident) => {
        $msg.react($ctx, serenity::model::prelude::ReactionType::from('❌')).await?;
    };
    ($ctx:ident, $msg:ident, $single_message:expr) => {{
        $msg.reply($ctx, $single_message).await?;
        $crate::failure!($ctx, $msg);
    }};
    ($ctx:ident, $msg:ident, $($error:tt)*) => {{
        $msg.reply($ctx, format!($($error)*)).await?;
        $crate::failure!($ctx, $msg);
    }};
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
        $crate::is_admin!($ctx, $msg.guild_id.unwrap_or_default(), $msg.author.id)
    };
    ($ctx:ident, $guild_id:expr, $user:expr) => {
        $crate::database::admin_data::is_admin_function($ctx, $guild_id, $user)
            .await
            .unwrap_or_default()
    };
}

/// Checks a [`User`]'s permissions.
///
/// Returns `true` if `user` has any of the [permissions][Permissions] `perm` in the
/// `guild`. If the user lacks the permissions,
/// returns `false`.
pub async fn has_permission(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
    perm: Permissions,
) -> bool {
    if let Some(member) = ctx.cache.member(guild_id, user_id) {
        return member.permissions(ctx).unwrap_or_default().intersects(perm);
    }

    false
}

pub fn parse_motd(motd: impl ToString) -> String {
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

pub fn to_json_safe_string(s: impl ToString) -> String {
    // serialize as string to get string escapes
    let s = serde_json::ser::to_string(&serde_json::Value::String(s.to_string())).unwrap();
    // remove the surrounding quotes
    s[1..s.len() - 1].to_string()
}

#[cfg(test)]
mod tests {
    use super::to_json_safe_string;

    #[test]
    fn test_json_safe_string() {
        let s = "\"holà\"\n}";

        assert_eq!(to_json_safe_string(s), "\\\"holà\\\"\\n}");
    }
}
