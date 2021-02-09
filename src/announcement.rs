use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::ChannelId;
use serenity::utils::Color;

pub async fn announce(ctx: &Context, channel: ChannelId, message: Value) -> CommandResult {
    let content = message["content"].as_str();
    let image = message["image"].as_str();
    let embed = &message["embed"];
    channel
        .send_message(ctx, |m| {
            if let Some(content) = content {
                m.content(content);
            }
            if let Some(image) = image {
                m.add_file(image);
            }
            if embed.is_object() {
                let colour = embed["colour"].as_str();
                let color = embed["color"].as_str();
                let author = &embed["author"];
                let title = embed["title"].as_str();
                let url = embed["url"].as_str();
                let description = embed["description"].as_str();
                let fields = embed["fields"].as_array();
                let image = embed["image"].as_str();
                let thumb = embed["thumbnail"].as_str();
                let footer = &embed["footer"];
                let timestamp = embed["timestamp"].as_str();
                m.embed(|e| {
                    if let Some(colour) = colour {
                        if let Ok(c) = u32::from_str_radix(&colour.to_uppercase(), 16) {
                            e.colour(Color::new(c));
                        }
                    } else if let Some(color) = color {
                        if let Ok(c) = u32::from_str_radix(&color.to_uppercase(), 16) {
                            e.colour(Color::new(c));
                        }
                    }
                    if author.is_object() {
                        let name = author["name"].as_str();
                        let icon = author["icon"].as_str();
                        let url = author["url"].as_str();
                        e.author(|a| {
                            if let Some(name) = name {
                                a.name(name);
                            }
                            if let Some(icon) = icon {
                                a.icon_url(icon);
                            }
                            if let Some(url) = url {
                                a.url(url);
                            }
                            a
                        });
                    }
                    if let Some(title) = title {
                        e.title(title);
                    }
                    if let Some(url) = url {
                        e.url(url);
                    }
                    if let Some(description) = description {
                        e.description(description);
                    }
                    if let Some(image) = image {
                        e.image(image);
                    }
                    if let Some(fields) = fields {
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
                    if let Some(thumb) = thumb {
                        e.thumbnail(thumb);
                    }
                    if footer.is_object() {
                        let icon = footer["icon"].as_str();
                        let text = footer["text"].as_str();
                        e.footer(|f| {
                            if let Some(icon) = icon {
                                f.icon_url(icon);
                            }
                            if let Some(text) = text {
                                f.text(text);
                            }
                            f
                        });
                    }
                    if let Some(timestamp) = timestamp {
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
