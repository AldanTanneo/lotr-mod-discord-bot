use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::id::ChannelId;

pub async fn announce(ctx: &Context, channel: ChannelId, content_json: &str) -> CommandResult {
    let message: Value = serde_json::from_str(content_json)?;
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
                let author = &embed["author"];
                let title = embed["title"].as_str();
                let description = embed["description"].as_str();
                let image = embed["image"].as_str();
                let fields = embed["fields"].as_array();
                m.embed(|e| {
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
                            if title.is_some() && content.is_some() && inlined.is_some() {
                                e.field(title.unwrap(), content.unwrap(), inlined.unwrap());
                            }
                        }
                    }
                    e
                });
            }
            m
        })
        .await?;
    Ok(())
}
