use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use crate::api::minecraft::get_server_status;
use crate::check::*;
use crate::database::config::{delete_minecraft_ip, get_minecraft_ip, set_minecraft_ip};
use crate::{failure, success};

#[command]
#[only_in(guilds)]
#[aliases("ip")]
#[bucket = "basic"]
#[sub_commands(set_ip, remove_ip)]
#[checks(is_minecraft_server)]
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
        failure!(
            ctx,
            msg,
            "No registered Minecraft IP for this server. Set one using `ip set <server ip>`."
        );
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[aliases("set")]
pub async fn set_ip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(ip) = args.single_quoted::<String>() {
        println!("Setting up IP");
        let update = get_minecraft_ip(ctx, msg.guild_id).await.is_some();
        set_minecraft_ip(ctx, msg.guild_id, &ip, update).await?;
        success!(ctx, msg, "Set Minecraft server IP to  `{}`", ip);
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[aliases("remove", "unset")]
pub async fn remove_ip(ctx: &Context, msg: &Message) -> CommandResult {
    let ip = get_minecraft_ip(ctx, msg.guild_id).await;
    if let Some(ip) = ip {
        delete_minecraft_ip(ctx, msg.guild_id).await?;
        success!(
            ctx,
            msg,
            "Successfully removed ip  `{}`  from this server",
            ip
        );
    } else {
        failure!(ctx, msg, "No registered Minecraft IP for this server.");
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_minecraft_server)]
#[bucket = "basic"]
pub async fn online(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let ip = if !args.is_empty() {
        args.single::<String>()?
    } else if let Some(ip) = get_minecraft_ip(ctx, msg.guild_id).await {
        ip
    } else {
        failure!(
            ctx,
            msg,
            "No registered Minecraft IP for this server. Set one using `ip set <server ip>`."
        );
        return Ok(());
    };
    println!("Getting status for ip: \"{}\"", ip);
    let server = get_server_status(&ip).await;
    if let Some(server) = server {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::DARK_GREEN);
                    e.thumbnail(format!("https://eu.mc-api.net/v3/server/favicon/{}", &ip));
                    e.title("Server online!");
                    e.description(format!("**{}**\n\n**IP:**  `{}`", &server.motd, &ip,));
                    e.field(
                        format!(
                            "Players: {}/{}",
                            &server.online_players, &server.max_players
                        ),
                        &server
                            .player_sample
                            .as_ref()
                            .map(|s| {
                                let res = s.join(", ").replace("_", "\\_");
                                if res.len() > 1024 {
                                    "Too many usernames to display!".into()
                                } else if res.is_empty() {
                                    "[]()".into()
                                } else {
                                    res
                                }
                            })
                            .unwrap_or_else(|| "[]()".into()),
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
