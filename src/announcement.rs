use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{id::ChannelId, prelude::ReactionType};
use serenity::utils::Color;
use std::convert::TryFrom;

pub async fn announce(ctx: &Context, channel: ChannelId, message: Value) -> CommandResult {
    channel
        .send_message(ctx, |m| {
            if let Some(content) = message["content"].as_str() {
                m.content(content);
            }
            if let Some(image) = message["image"].as_str() {
                m.add_file(image);
            }
            if let Some(reactions) = message["reactions"].as_array() {
                m.reactions(
                    reactions
                        .iter()
                        .filter_map(|s| s.as_str())
                        .filter_map(|s| ReactionType::try_from(s).ok()),
                );
            }
            if message["embed"].is_object() {
                let embed = &message["embed"];
                m.embed(|e| {
                    if let Some(colour) = embed["colour"].as_str() {
                        if let Ok(c) = u32::from_str_radix(&colour.to_uppercase(), 16) {
                            e.colour(Color::new(c));
                        }
                    } else if let Some(color) = embed["color"].as_str() {
                        if let Ok(c) = u32::from_str_radix(&color.to_uppercase(), 16) {
                            e.colour(Color::new(c));
                        }
                    }
                    if embed["author"].is_object() {
                        let author = &embed["author"];
                        e.author(|a| {
                            if let Some(name) = author["name"].as_str() {
                                a.name(name);
                            }
                            if let Some(icon) = author["icon"].as_str() {
                                a.icon_url(icon);
                            }
                            if let Some(url) = author["url"].as_str() {
                                a.url(url);
                            }
                            a
                        });
                    }
                    if let Some(title) = embed["title"].as_str() {
                        e.title(title);
                    }
                    if let Some(url) = embed["url"].as_str() {
                        e.url(url);
                    }
                    if let Some(description) = embed["description"].as_str() {
                        e.description(description);
                    }
                    if let Some(image) = embed["image"].as_str() {
                        e.image(image);
                    }
                    if let Some(fields) = embed["fields"].as_array() {
                        for field in fields {
                            let title = field[0].as_str();
                            let content = field[1].as_str();
                            let inlined = field[2].as_bool();
                            if let (Some(title), Some(content), Some(inlined)) =
                                (title, content, inlined)
                            {
                                e.field(title, content, inlined);
                            }
                        }
                    }
                    if let Some(thumb) = embed["thumbnail"].as_str() {
                        e.thumbnail(thumb);
                    }
                    if embed["footer"].is_object() {
                        let footer = &embed["footer"];
                        e.footer(|f| {
                            if let Some(icon) = footer["icon"].as_str() {
                                f.icon_url(icon);
                            }
                            if let Some(text) = footer["text"].as_str() {
                                f.text(text);
                            }
                            f
                        });
                    }
                    if let Some(timestamp) = embed["timestamp"].as_str() {
                        e.timestamp(timestamp);
                    }
                    e
                });
            }
            m
        })
        .await?;
    Ok(())
}
