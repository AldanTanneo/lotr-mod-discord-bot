use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use crate::api::minecraft::get_server_status;
use crate::check::IS_ADMIN_CHECK;
use crate::database::config::{get_minecraft_ip, set_minecraft_ip};

#[command]
#[aliases("ip")]
#[bucket = "basic"]
#[sub_commands(set_ip)]
#[only_in(guilds)]
async fn server_ip(ctx: &Context, msg: &Message) -> CommandResult {
    let ip = get_minecraft_ip(ctx, msg.guild_id).await;
    if let Some(ip) = ip {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::TEAL);
                    e.title("Server IP:");
                    e.description(format!("`{}`", ip));
                    e
                })
            })
            .await?;
    } else {
        msg.reply(ctx, "No registered Minecraft IP for this server.")
            .await?;
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("set")]
#[only_in(guilds)]
pub async fn set_ip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(ip) = args.single_quoted() {
        println!("Setting up IP");
        let update = get_minecraft_ip(ctx, msg.guild_id).await.is_some();
        set_minecraft_ip(ctx, msg.guild_id, &ip, update).await?;
        msg.reply(ctx, format!("Set Minecraft server IP to \"{}\"", &ip))
            .await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn online(ctx: &Context, msg: &Message) -> CommandResult {
    let ip = if let Some(ip) = get_minecraft_ip(ctx, msg.guild_id).await {
        ip
    } else {
        msg.reply(ctx, "No registered Minecraft IP for this server.")
            .await?;
        return Ok(());
    };
    let server = get_server_status(ctx, &ip).await;
    if let Some(server) = server {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::DARK_GREEN);
                    e.thumbnail(format!("https://eu.mc-api.net/v3/server/favicon/{}", &ip));
                    e.title("Server online!");
                    e.description(format!(
                        "**{}**\n\n**IP:**  `{}`",
                        &server.motd.clean.join("\n"),
                        &ip,
                    ));
                    let on = server.players.online;
                    e.field(
                        format!(
                            "Players: {}/{}",
                            &server.players.online, &server.players.max
                        ),
                        format!(
                            "{}",
                            &server
                                .players
                                .list
                                .map(|s| s.join(", ").replace("_", "\\_"))
                                .unwrap_or_else(|| "[]()".into())
                        ),
                        false,
                    );
                    e
                });
                m.reference_message(msg);
                m.allowed_mentions(|a| a.empty_parse());
                m
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::RED);
                    e.title("Server offline...");
                    e.description(format!("**IP:**  `{}`", &ip));
                    e
                });
                m.reference_message(msg);
                m.allowed_mentions(|a| a.empty_parse());
                m
            })
            .await?;
    }
    Ok(())
}
