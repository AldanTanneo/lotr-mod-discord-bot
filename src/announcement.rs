//! Announcement function [`announce`] that posts a JSON message

use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::futures::future::join_all;
use serenity::model::id::{ChannelId, MessageId};
use serenity::model::prelude::ReactionType;
use serenity::utils::Color;
use std::convert::TryFrom;

/// Macro that builds an embed. Used in [`announce`] and [`edit_message`].
macro_rules! embed_parser {
    ($m:ident, $message:ident) => {
        let embed = &$message["embed"];
        $m.embed(|e| {
            if let Some(colour) = embed["colour"].as_str() {
                // embed side bar colour
                // (hexadecimal color encoding)
                if let Ok(c) = u32::from_str_radix(&colour.to_uppercase(), 16) {
                    e.colour(Color::new(c));
                }
            } else if let Some(color) = embed["color"].as_str() {
                // supports english and american spelling
                if let Ok(c) = u32::from_str_radix(&color.to_uppercase(), 16) {
                    e.colour(Color::new(c));
                }
            }
            if embed["author"].is_object() {
                // embed author
                let author = &embed["author"];
                e.author(|a| {
                    if let Some(name) = author["name"].as_str() {
                        // author name
                        a.name(name);
                    }
                    if let Some(icon) = author["icon"].as_str() {
                        // author icon url
                        a.icon_url(icon);
                    }
                    if let Some(url) = author["url"].as_str() {
                        // author clickable link
                        a.url(url);
                    }
                    a
                });
            }
            if let Some(title) = embed["title"].as_str() {
                // embed title
                e.title(title);
            }
            if let Some(url) = embed["url"].as_str() {
                // title clickable link
                e.url(url);
            }
            if let Some(description) = embed["description"].as_str() {
                // embed description, displays smaller than fields,
                // but allows for longer text
                e.description(description);
            }
            if let Some(image) = embed["image"].as_str() {
                // embed image url
                e.image(image);
            }
            if let Some(fields) = embed["fields"].as_array() {
                // fields array, filters out invalid fields
                for field in fields {
                    let title = field[0].as_str();
                    let content = field[1].as_str();
                    // defaults to not inlined field
                    let inlined = field[2].as_bool().unwrap_or(false);
                    if let (Some(title), Some(content)) = (title, content) {
                        e.field(title, content, inlined);
                    }
                }
            }
            if let Some(field) = embed["field"].as_array() {
                // single field, when multiple are not needed
                let title = field[0].as_str();
                let content = field[1].as_str();
                // still defaults to not inlined field
                let inlined = field[2].as_bool().unwrap_or(false);
                if let (Some(title), Some(content)) = (title, content) {
                    e.field(title, content, inlined);
                }
            }
            if let Some(thumb) = embed["thumbnail"].as_str() {
                // embed thumbnail url
                e.thumbnail(thumb);
            }
            if embed["footer"].is_object() {
                // embed footer
                let footer = &embed["footer"];
                e.footer(|f| {
                    if let Some(icon) = footer["icon"].as_str() {
                        // footer icon url
                        f.icon_url(icon);
                    }
                    if let Some(text) = footer["text"].as_str() {
                        // footer text
                        f.text(text);
                    }
                    f
                });
            }
            if let Some(timestamp) = embed["timestamp"].as_str() {
                // embed timestamp, displays the date right after the
                // footer
                e.timestamp(timestamp);
            }
            e
        });
    };
}

/// Parser function that will post a JSON `message` in the right `channel`.
///
/// # Example
///
/// ```
/// use serenity::model::id::ChannelId;
///
/// let channel = ChannelId(7);
/// let data = r#"
///     {
///        "content": "Some content",
///        "reactions": ['❌', '️️️️️️️️❤️']
///     }"#;
/// let message: Value = serde_json::from_str(&data).unwrap();
///
/// announce(ctx, channel, message).await;
/// ```
///
/// # JSON Documentation
///
/// The JSON message supports a lot of fields, that you can find here:
/// ```json
/// {
///     "content": "the message content",
///     "image": "a valid image url",
///     "reactions": [
///         "🍎", // unicode emojis
///         "<:name:0000000000000000>" // custom emojis
///     ],
///     "embed": {
///         "colour": "RRGGBB", // hexadecimal color code
///         "author": {
///             "name": "the embed author name",
///             "icon": "a valid author icon url",
///             "url": "a valid url that will open when clicking on the author name"
///         },
///         "title": "the embed title",
///         "url": "a valid url that will open when clicking on the title",
///         "description": "the embed description",
///         "image": "an embed image",
///         "thumbnail": "a valid thumbnail image url",
///         "fields": [ // a list of fields to display in the embed; an element looks like:
///             [
///                 "a field title",
///                 "some field content",
///                 true // or false: wether the field is inlined or not
///                      // (if not, displays as a block)
///                      // [defaults to false]
///             ]
///         ],
///         "field": [ // single field, when multiple are not needed
///             "a field title",
///             "some field content",
///             true // or false: wether the field is inlined or not
///                  // (if not, displays as a block)
///                  // [defaults to false]
///         ],
///         "footer" : {
///             "icon": "a valid footer icon url",
///             "text": "some footer text"
///         },
///         "timestamp": "a valid timestamp in the format [YYYY]-[MM]-[DD]T[HH]:[mm]:[ss]"
///                     // example: "2020-12-02T13:07:00"
///     }
/// }
/// ```
pub async fn announce(ctx: &Context, channel: ChannelId, message: Value) -> CommandResult {
    channel
        .send_message(ctx, |m| {
            if let Some(content) = message["content"].as_str() {
                // main message content
                m.content(content);
            }
            if let Some(image) = message["image"].as_str() {
                // message attachment url (doesn't have to be an image)
                m.add_file(image);
            }
            if let Some(reactions) = message["reactions"].as_array() {
                // reactions array, filters out invalid reactions
                m.reactions(
                    reactions
                        .iter()
                        .filter_map(|s| s.as_str())
                        .filter_map(|s| ReactionType::try_from(s).ok()),
                );
            }
            if message["embed"].is_object() {
                // message embed content
                embed_parser!(m, message);
            }
            m
        })
        .await?;
    Ok(())
}

/// Editing function that allows for editing a message posted with
/// [`announce`]
///
/// The `message` JSON content supports all the fields of the [`announce`] function,
/// with the exception of `"image"`.
pub async fn edit_message(
    ctx: &Context,
    channel: ChannelId,
    msg_id: MessageId,
    message: Value,
) -> CommandResult {
    let msg = channel
        .edit_message(ctx, msg_id, |m| {
            if let Some(content) = message["content"].as_str() {
                // main message content
                m.content(content);
            }
            if message["embed"].is_object() {
                // message embed content
                embed_parser!(m, message);
            }
            m
        })
        .await?;
    if let Some(reactions) = message["reactions"].as_array() {
        join_all(
            reactions
                .iter()
                .filter_map(|s| s.as_str())
                .filter_map(|s| ReactionType::try_from(s).ok())
                .map(|r| msg.react(ctx, r)),
        )
        .await;
    }
    Ok(())
}
