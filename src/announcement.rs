//! Announcement function [`announce`] that posts a JSON message

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serenity::builder::{CreateEmbed, CreateMessage, EditMessage};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::futures::future::join_all;
use serenity::model::application::component::ButtonStyle;
use serenity::model::prelude::*;
use serenity::utils::Colour;
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementEmbedAuthorPreset {
    LotrFacebook,
    LotrInstagram,
    Mevans,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AnnouncementEmbedAuthor {
    Object {
        name: String,
        url: Option<String>,
        #[serde(alias = "icon_url")]
        icon: Option<String>,
    },
    Preset(AnnouncementEmbedAuthorPreset),
}

fn deserialize_embed_colour<'de, D>(d: D) -> Result<Colour, D::Error>
where
    D: Deserializer<'de>,
{
    use AnnouncementError::InvalidColour;

    let colour = String::deserialize(d)?;

    if let Ok(c) = u32::from_str_radix(colour.trim().trim_start_matches('#'), 16) {
        Ok(Colour(c))
    } else if let Some(web_colour) =
        crate::utils::parse_web_colour(colour.trim().to_ascii_lowercase().as_str())
    {
        Ok(web_colour)
    } else {
        Err(serde::de::Error::custom(InvalidColour(colour)))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "Colour")]
pub struct AnnouncementEmbedColour(
    #[serde(deserialize_with = "deserialize_embed_colour")] pub Colour,
);

impl From<AnnouncementEmbedColour> for Colour {
    fn from(c: AnnouncementEmbedColour) -> Colour {
        c.0
    }
}

#[derive(Debug, serde_tuple::Serialize_tuple, serde_tuple::Deserialize_tuple, Clone)]
pub struct AnnouncementEmbedField {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub inlined: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnouncementEmbedFooter {
    pub text: String,
    #[serde(alias = "icon_url")]
    pub icon: Option<String>,
}

fn deserialize_iso8601<'de, D>(de: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use iso_8601::{ApproxDate, ApproxGlobalTime};
    use std::str::FromStr;

    let s = <&str>::deserialize(de)?;

    iso_8601::DateTime::<ApproxDate, ApproxGlobalTime>::from_str(s)
        .map_err(serde::de::Error::custom)
        .map(chrono::DateTime::<Utc>::from)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "DateTime<Utc>")]
pub struct AnnouncementEmbedTimestamp(
    #[serde(deserialize_with = "deserialize_iso8601")] pub DateTime<Utc>,
);

impl From<AnnouncementEmbedTimestamp> for DateTime<Utc> {
    fn from(ts: AnnouncementEmbedTimestamp) -> DateTime<Utc> {
        ts.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnouncementEmbed {
    pub author: Option<AnnouncementEmbedAuthor>,
    #[serde(alias = "color", skip_serializing_if = "Option::is_none")]
    pub colour: Option<AnnouncementEmbedColour>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub thumbnail: Option<String>,

    pub field: Option<AnnouncementEmbedField>,
    pub fields: Option<Vec<AnnouncementEmbedField>>,

    pub footer: Option<AnnouncementEmbedFooter>,
    pub timestamp: Option<AnnouncementEmbedTimestamp>,
}

fn deserialize_announcement_reaction<'de, D>(d: D) -> Result<ReactionType, D::Error>
where
    D: Deserializer<'de>,
{
    use AnnouncementError::InvalidReaction;

    let reaction = String::deserialize(d)?;

    ReactionType::try_from(reaction.as_str())
        .map_err(|_| serde::de::Error::custom(InvalidReaction(reaction)))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "String")]
pub struct AnnouncementReaction(
    #[serde(deserialize_with = "deserialize_announcement_reaction")] pub ReactionType,
);

impl From<AnnouncementReaction> for String {
    fn from(AnnouncementReaction(reaction): AnnouncementReaction) -> Self {
        reaction.to_string()
    }
}

impl From<&AnnouncementReaction> for ReactionType {
    fn from(r: &AnnouncementReaction) -> Self {
        r.0.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnouncementButton {
    pub url: String,
    pub label: Option<String>,
    pub emoji: Option<AnnouncementReaction>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Announcement {
    pub content: Option<String>,
    #[serde(alias = "image")]
    pub file: Option<String>,
    #[serde(alias = "images")]
    pub files: Option<Vec<String>>,

    pub embed: Option<AnnouncementEmbed>,
    pub embeds: Option<Vec<AnnouncementEmbed>>,
    pub delete_embeds: Option<bool>,

    pub reactions: Option<Vec<AnnouncementReaction>>,
    pub link_buttons: Option<Vec<AnnouncementButton>>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum AnnouncementError {
    InvalidColour(String),
    InvalidReaction(String),
}

impl std::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use AnnouncementError::*;

        match self {
            InvalidColour(c) => write!(
                f,
                "invalid colour, not a valid hexadecimal value or web colour name: `{c}`"
            ),
            InvalidReaction(e) => {
                write!(
                    f,
                    "reaction conversion failed on invalid reaction string: `{e}`"
                )
            }
        }
    }
}

impl std::error::Error for AnnouncementError {}

fn parse_embed(embed: &AnnouncementEmbed) -> CreateEmbed {
    use AnnouncementEmbedAuthor::*;

    let mut builder = CreateEmbed::default();

    match &embed.author {
        Some(Object { name, url, icon }) => {
            builder.author(|a| {
                if let Some(url) = url {
                    a.url(url);
                }
                if let Some(icon) = icon {
                    a.icon_url(icon);
                }
                a.name(name)
            });
        }
        Some(Preset(preset)) => {
            use AnnouncementEmbedAuthorPreset::*;
            match preset {
                LotrFacebook => {
                    builder.author(|a| {
                        a.name("LOTR Mod Official Facebook");
                        a.url("https://www.facebook.com/LOTRMC");
                        a.icon_url(crate::constants::FACEBOOK_ICON);
                        a
                    });
                    builder.colour(crate::constants::FACEBOOK_COLOUR);
                }
                LotrInstagram => {
                    builder.author(|a| {
                        a.name("LOTR Mod Official Instagram");
                        a.url("https://www.instagram.com/lotrmcmod");
                        a.icon_url(crate::constants::INSTAGRAM_ICON);
                        a
                    });
                    builder.colour(crate::constants::INSTAGRAM_COLOUR);
                }
                Mevans => {
                    builder.author(|a| {
                        a.name("Mevans");
                        a.icon_url("https://cdn.discordapp.com/emojis/405159804127150090.png");
                        a
                    });
                }
            }
        }
        _ => (),
    }

    if let Some(AnnouncementEmbedColour(colour)) = embed.colour {
        builder.colour(colour);
    }

    if let Some(title) = &embed.title {
        // embed title
        builder.title(title);
    }
    if let Some(url) = &embed.url {
        // title clickable link
        builder.url(url);
    }
    if let Some(description) = &embed.description {
        // embed description, displays smaller than fields,
        // but allows for longer text
        builder.description(description);
    }

    if let Some(image) = &embed.image {
        // embed image url
        builder.image(image);
    }
    if let Some(fields) = &embed.fields {
        // fields array, filters out invalid fields
        for AnnouncementEmbedField {
            title,
            content,
            inlined,
        } in fields
        {
            builder.field(title, content, *inlined);
        }
    }
    if let Some(AnnouncementEmbedField {
        title,
        content,
        inlined,
    }) = &embed.field
    {
        builder.field(title, content, *inlined);
    }

    if let Some(thumbnail) = &embed.thumbnail {
        // embed thumbnail url
        builder.thumbnail(thumbnail);
    }

    if let Some(AnnouncementEmbedFooter { text, icon }) = &embed.footer {
        builder.footer(|f| {
            if let Some(icon) = icon {
                f.icon_url(icon);
            }
            // footer text
            f.text(text)
        });
    }

    if let Some(timestamp) = &embed.timestamp {
        builder.timestamp(timestamp.0);
    }

    builder
}

pub async fn announce(ctx: &Context, channel: ChannelId, message: &Announcement) -> CommandResult {
    let mut builder = CreateMessage::default();

    // message content
    if let Some(content) = &message.content {
        builder.content(content);
    }

    // attachements
    if let Some(file) = &message.file {
        builder.add_file(file.as_str());
    }
    if let Some(files) = &message.files {
        for file in files {
            builder.add_file(file.as_str());
        }
    }

    // reactions
    if let Some(reactions) = &message.reactions {
        // reactions array, filters out invalid reactions
        builder.reactions(reactions);
    }

    // embeds
    if let Some(embeds) = &message.embeds {
        for embed in embeds {
            builder.add_embed(|e| {
                *e = parse_embed(embed);
                e
            });
        }
    }
    if let Some(embed) = &message.embed {
        // message embed content
        builder.set_embed(parse_embed(embed));
    }

    // components
    if let Some(buttons) = &message.link_buttons {
        builder.components(|c| {
            if !buttons.is_empty() {
                c.create_action_row(|a| {
                    for button in buttons {
                        a.create_button(|b| {
                            b.style(ButtonStyle::Link)
                                .url(&button.url)
                                .disabled(button.disabled);
                            if let Some(label) = &button.label {
                                b.label(label);
                            }
                            if let Some(emoji) = &button.emoji {
                                b.emoji(emoji.0.clone());
                            }
                            b
                        });
                    }

                    a
                });
            }
            c
        });
    }

    channel
        .send_message(ctx, |m| {
            *m = builder;
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
    message: &Announcement,
) -> CommandResult {
    let mut builder = EditMessage::default();

    if let Some(content) = &message.content {
        // main message content
        builder.content(content);
    }
    if let Some(supress) = message.delete_embeds {
        builder.suppress_embeds(supress);
    }

    if let Some(embeds) = &message.embeds {
        for embed in embeds {
            builder.add_embed(|e| {
                *e = parse_embed(embed);
                e
            });
        }
    }
    if let Some(embed) = &message.embed {
        // message embed content
        builder.set_embed(parse_embed(embed));
    }

    if let Some(buttons) = &message.link_buttons {
        builder.components(|c| {
            if !buttons.is_empty() {
                c.create_action_row(|a| {
                    for button in buttons {
                        a.create_button(|b| {
                            b.style(ButtonStyle::Link)
                                .url(&button.url)
                                .disabled(button.disabled);
                            if let Some(label) = &button.label {
                                b.label(label);
                            }
                            if let Some(emoji) = &button.emoji {
                                b.emoji(emoji.0.clone());
                            }
                            b
                        });
                    }

                    a
                });
            }
            c
        });
    }

    let msg = channel
        .edit_message(ctx, msg_id, |m| {
            *m = builder;
            m
        })
        .await?;

    if let Some(reactions) = &message.reactions {
        join_all(reactions.iter().map(|r| msg.react(ctx, r))).await;
    }

    Ok(())
}
