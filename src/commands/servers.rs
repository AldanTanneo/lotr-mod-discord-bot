use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use crate::api::minecraft::get_server_status;
use crate::check::*;
use crate::database::config::{delete_minecraft_ip, get_minecraft_ip, set_minecraft_ip};
use crate::utils::{parse_motd, NotInGuild};
use crate::{failure, success};

#[command]
#[only_in(guilds)]
#[aliases("ip")]
#[bucket = "basic"]
#[sub_commands(set_ip, remove_ip)]
#[checks(is_minecraft_server)]
async fn server_ip(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(ip) = get_minecraft_ip(ctx, server_id).await {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::TEAL);
                    e.title("Server IP:");
                    e.description(format!("`{ip}`"));
                    e
                })
            })
            .await?;
    } else {
        failure!(
            ctx,
            msg,
            "No registered Minecraft IP for this server. Set one using  `!ip set <server ip>`."
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[aliases("set")]
pub async fn set_ip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(ip) = args.current() {
        println!("Setting up IP to {ip} on {server_id}");
        set_minecraft_ip(ctx, server_id, ip).await?;
        success!(ctx, msg, "Set Minecraft server IP to  `{}`", ip);
    } else {
        failure!(ctx, msg, "You must provide an IP address to set.");
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[aliases("remove", "unset")]
pub async fn remove_ip(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let ip = get_minecraft_ip(ctx, server_id).await;
    if let Some(ip) = ip {
        delete_minecraft_ip(ctx, server_id).await?;
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
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let ip = if !args.is_empty() {
        args.single::<String>()?
    } else if let Some(ip) = get_minecraft_ip(ctx, server_id).await {
        ip
    } else {
        failure!(
            ctx,
            msg,
            "No registered Minecraft IP for this server. Set one using  `!ip set <server ip>`."
        );
        return Ok(());
    };
    let server = get_server_status(ctx, &ip).await;
    if let Some(server) = server {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::DARK_GREEN);
                    e.thumbnail(format!("https://api.mcstatus.io/v2/icon/{}", &ip));
                    e.title("Server online!");
                    e.description(format!(
                        "{}\n\n**IP:**  `{}`",
                        parse_motd(server.motd.raw.join("\n")),
                        &ip,
                    ));

                    let (lst, first) = server
                        .players
                        .list
                        .into_iter()
                        .map(|p| p.replace('_', "\\_"))
                        .fold((String::new(), true), |(mut curr, first), p| {
                            if curr.len() + curr.len().min(2) + p.len() >= 1024 {
                                let title = if first {
                                    format!(
                                        "Players: {}/{}",
                                        server.players.online, server.players.max
                                    )
                                } else {
                                    String::new()
                                };
                                e.field(title, curr, false);
                                (p, false)
                            } else {
                                if !curr.is_empty() {
                                    curr.push_str(", ");
                                }
                                curr.push_str(&p);
                                (curr, first)
                            }
                        });

                    let title = if first {
                        format!("Players: {}/{}", server.players.online, server.players.max)
                    } else {
                        String::new()
                    };
                    e.field(title, lst, false);

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
                    e.description(format!("**IP:**  `{ip}`"));
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
