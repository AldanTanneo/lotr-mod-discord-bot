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

impl std::fmt::Display for JsonMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use JsonMessageError::*;

        match self {
            FileTooBig(size) => {
                write!(f, "File too big to download: {}", bytesize::ByteSize(*size))
            }
            DownloadError(e) => write!(f, "Could not download attachment: {e}"),
            JsonError(e) => write!(f, "Error reading JSON content: {e}"),
        }
    }
}

impl std::error::Error for JsonMessageError {}

pub async fn get_json_from_message<T: DeserializeOwned>(
    msg: &Message,
) -> Result<T, JsonMessageError> {
    use JsonMessageError::*;

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
    if ctx
        .cache
        .guild_field(guild_id, |g| g.owner_id == user_id)
        .unwrap_or_default()
    {
        return true;
    }

    if let Some(member) = ctx.cache.member(guild_id, user_id) {
        return member.permissions(ctx).unwrap_or_default().intersects(perm);
    } else if let Ok(member) = guild_id.member(ctx, user_id).await {
        return member.permissions(ctx).unwrap_or_default().intersects(perm);
    }

    false
}

pub fn parse_motd(motd: impl AsRef<str>) -> String {
    let motd = motd.as_ref();
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
                    res.push(c);
                }
            }
        } else {
            res.push(c);
        }
    }
    stack.drain(..).rev().for_each(|t| res.push_str(t));
    res
}

pub fn to_json_safe_string(s: impl AsRef<str>) -> String {
    // serialize as string to get string escapes
    let s = serde_json::ser::to_string(s.as_ref()).unwrap();
    // remove the surrounding quotes
    s[1..s.len() - 1].to_string()
}

use serenity::utils::Colour;

pub fn parse_web_colour(name: &str) -> Option<Colour> {
    Some(Colour(match name {
        "aliceblue" => 0xf0f8ff,
        "antiquewhite" => 0xfaebd7,
        "aqua" | "cyan" => 0x00ffff,
        "aquamarine" => 0x7fffd4,
        "azure" => 0xf0ffff,
        "beige" => 0xf5f5dc,
        "bisque" => 0xffe4c4,
        "black" => 0x000000,
        "blanchedalmond" => 0xffebcd,
        "blue" => 0x0000ff,
        "blueviolet" => 0x8a2be2,
        "brown" => 0xa52a2a,
        "burlywood" => 0xdeb887,
        "cadetblue" => 0x5f9ea0,
        "chartreuse" => 0x7fff00,
        "chocolate" => 0xd2691e,
        "coral" => 0xff7f50,
        "cornflowerblue" => 0x6495ed,
        "cornsilk" => 0xfff8dc,
        "crimson" => 0xdc143c,
        "darkblue" => 0x00008b,
        "darkcyan" => 0x008b8b,
        "darkgoldenrod" => 0xb8860b,
        "darkgray" | "darkgrey" => 0xa9a9a9,
        "darkgreen" => 0x006400,
        "darkkhaki" => 0xbdb76b,
        "darkmagenta" => 0x8b008b,
        "darkolivegreen" => 0x556b2f,
        "darkorange" => 0xff8c00,
        "darkorchid" => 0x9932cc,
        "darkred" => 0x8b0000,
        "darksalmon" => 0xe9967a,
        "darkseagreen" => 0x8fbc8f,
        "darkslateblue" => 0x483d8b,
        "darkslategray" | "darkslategrey" => 0x2f4f4f,
        "darkturquoise" => 0x00ced1,
        "darkviolet" => 0x9400d3,
        "deeppink" => 0xff1493,
        "deepskyblue" => 0x00bfff,
        "dimgray" | "dimgrey" => 0x696969,
        "dodgerblue" => 0x1e90ff,
        "firebrick" => 0xb22222,
        "floralwhite" => 0xfffaf0,
        "forestgreen" => 0x228b22,
        "fuchsia" | "magenta" => 0xff00ff,
        "gainsboro" => 0xdcdcdc,
        "ghostwhite" => 0xf8f8ff,
        "gold" => 0xffd700,
        "goldenrod" => 0xdaa520,
        "gray" | "grey" => 0x808080,
        "green" => 0x008000,
        "greenyellow" => 0xadff2f,
        "honeydew" => 0xf0fff0,
        "hotpink" => 0xff69b4,
        "indianred" => 0xcd5c5c,
        "indigo" => 0x4b0082,
        "ivory" => 0xfffff0,
        "khaki" => 0xf0e68c,
        "lavender" => 0xe6e6fa,
        "lavenderblush" => 0xfff0f5,
        "lawngreen" => 0x7cfc00,
        "lemonchiffon" => 0xfffacd,
        "lightblue" => 0xadd8e6,
        "lightcoral" => 0xf08080,
        "lightcyan" => 0xe0ffff,
        "lightgoldenrodyellow" => 0xfafad2,
        "lightgray" | "lightgrey" => 0xd3d3d3,
        "lightgreen" => 0x90ee90,
        "lightpink" => 0xffb6c1,
        "lightsalmon" => 0xffa07a,
        "lightseagreen" => 0x20b2aa,
        "lightskyblue" => 0x87cefa,
        "lightslategray" | "lightslategrey" => 0x778899,
        "lightsteelblue" => 0xb0c4de,
        "lightyellow" => 0xffffe0,
        "lime" => 0x00ff00,
        "limegreen" => 0x32cd32,
        "linen" => 0xfaf0e6,
        "maroon" => 0x800000,
        "mediumaquamarine" => 0x66cdaa,
        "mediumblue" => 0x0000cd,
        "mediumorchid" => 0xba55d3,
        "mediumpurple" => 0x9370db,
        "mediumseagreen" => 0x3cb371,
        "mediumslateblue" => 0x7b68ee,
        "mediumspringgreen" => 0x00fa9a,
        "mediumturquoise" => 0x48d1cc,
        "mediumvioletred" => 0xc71585,
        "midnightblue" => 0x191970,
        "mintcream" => 0xf5fffa,
        "mistyrose" => 0xffe4e1,
        "moccasin" => 0xffe4b5,
        "navajowhite" => 0xffdead,
        "navy" => 0x000080,
        "oldlace" => 0xfdf5e6,
        "olive" => 0x808000,
        "olivedrab" => 0x6b8e23,
        "orange" => 0xffa500,
        "orangered" => 0xff4500,
        "orchid" => 0xda70d6,
        "palegoldenrod" => 0xeee8aa,
        "palegreen" => 0x98fb98,
        "paleturquoise" => 0xafeeee,
        "palevioletred" => 0xdb7093,
        "papayawhip" => 0xffefd5,
        "peachpuff" => 0xffdab9,
        "peru" => 0xcd853f,
        "pink" => 0xffc0cb,
        "plum" => 0xdda0dd,
        "powderblue" => 0xb0e0e6,
        "purple" => 0x800080,
        "red" => 0xff0000,
        "rosybrown" => 0xbc8f8f,
        "royalblue" => 0x4169e1,
        "saddlebrown" => 0x8b4513,
        "salmon" => 0xfa8072,
        "sandybrown" => 0xf4a460,
        "seagreen" => 0x2e8b57,
        "seashell" => 0xfff5ee,
        "sienna" => 0xa0522d,
        "silver" => 0xc0c0c0,
        "skyblue" => 0x87ceeb,
        "slateblue" => 0x6a5acd,
        "slategray" | "slategrey" => 0x708090,
        "snow" => 0xfffafa,
        "springgreen" => 0x00ff7f,
        "steelblue" => 0x4682b4,
        "tan" => 0xd2b48c,
        "teal" => 0x008080,
        "thistle" => 0xd8bfd8,
        "tomato" => 0xff6347,
        "turquoise" => 0x40e0d0,
        "violet" => 0xee82ee,
        "wheat" => 0xf5deb3,
        "white" => 0xffffff,
        "whitesmoke" => 0xf5f5f5,
        "yellow" => 0xffff00,
        "yellowgreen" => 0x9acd32,
        _ => {
            return None;
        }
    }))
}

use serenity::async_trait;
use serenity::builder::CreateInteractionResponse;
use serenity::http::Http;
use serenity::model::application::interaction::{
    message_component::MessageComponentInteraction, MessageFlags,
};

#[async_trait]
pub trait InteractionEasyResponse {
    async fn say_ephemeral(
        &self,
        ctx: impl AsRef<Http> + Send + Sync + 'async_trait,
        msg: impl ToString + Send + Sync + 'async_trait,
    ) -> () {
        self.respond_no_failure(ctx, |r| {
            r.interaction_response_data(|d| d.content(msg).flags(MessageFlags::EPHEMERAL))
        })
        .await;
    }

    async fn respond_no_failure<F>(
        &self,
        ctx: impl AsRef<Http> + Send + Sync + 'async_trait,
        f: F,
    ) -> ()
    where
        for<'a, 'r> F: 'async_trait
            + Send
            + Sync
            + FnOnce(&'a mut CreateInteractionResponse<'r>) -> &'a mut CreateInteractionResponse<'r>;
}

#[async_trait]
impl InteractionEasyResponse for MessageComponentInteraction {
    async fn respond_no_failure<F>(
        &self,
        ctx: impl AsRef<Http> + Send + Sync + 'async_trait,
        f: F,
    ) -> ()
    where
        for<'a, 'r> F: 'async_trait
            + Send
            + Sync
            + FnOnce(&'a mut CreateInteractionResponse<'r>) -> &'a mut CreateInteractionResponse<'r>,
    {
        if let Err(e) = self.create_interaction_response(ctx, f).await {
            println!(
                "=== ERROR ===
Error sending component interaction response to {} {:?}
Error: {}
=== END ===",
                self.user.tag(),
                self.user.id,
                e
            );
        }
    }
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
