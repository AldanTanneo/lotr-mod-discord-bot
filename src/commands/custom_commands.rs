use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::announcement::announce;
use crate::check::*;
use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID, RESERVED_NAMES};
use crate::database::{
    blacklist::check_blacklist,
    custom_commands::{
        add_custom_command, check_command_exists, get_command_data, get_custom_commands_list,
        remove_custom_command,
    },
    Blacklist,
};
use crate::utils::{get_json_from_message, has_permission, NotInGuild};
use crate::{failure, handle_json_error, is_admin, success};

#[command]
#[only_in(guilds)]
#[aliases("command")]
#[sub_commands(define, custom_command_remove, custom_command_display)]
pub async fn custom_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    println!("Custom command execution...");
    let name = args.single::<String>()?.to_lowercase(); // getting command name
    let subcommand = args.current(); // getting possible subcommand but not advancing

    if let Some(command_data) = get_command_data(ctx, server_id, &name, false).await {
        let mut message: Value = serde_json::from_str(&command_data.body.replace("\\$", "$"))?;
        let mut delete = message["self_delete"].as_bool().unwrap_or_default();
        // early interrupt in case of blacklist / admin command
        let command_type = if subcommand.is_some()
            && message["subcommands"][subcommand.unwrap()]["type"].is_string()
        {
            // optionnally overriding the command type
            message["subcommands"][subcommand.unwrap()]["type"].as_str()
        } else {
            message["type"].as_str()
        };
        if let Some(s) = command_type {
            let is_admin = msg.author.id == OWNER_ID
                || is_admin!(ctx, msg)
                || has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await;
            if !is_admin {
                if s == "meme"
                    && check_blacklist(ctx, msg, false)
                        .await
                        .unwrap_or(Blacklist::IsBlacklisted(true))
                        .is_blacklisted()
                {
                    msg.delete(ctx).await?;
                    msg.author
                        .dm(ctx, |m| {
                            m.content("You are not allowed to use this command here!")
                        })
                        .await?;
                    return Ok(());
                } else if s == "admin" {
                    failure!(ctx, msg, "You are not an admin on this server!");
                }
            }
        }

        let mut command_body = command_data.body;
        if let Some(subcommand) = subcommand {
            if message["subcommands"][subcommand].is_object() {
                message = message["subcommands"][subcommand].clone();
                command_body = serde_json::to_string(&message)?;
                args.advance();
            } else if let Value::String(subcommand) = &message["subcommands"][subcommand] {
                if message["subcommands"][subcommand].is_object() {
                    message = message["subcommands"][subcommand].clone();
                    command_body = serde_json::to_string(&message)?;
                    args.advance();
                }
            }
        }
        if command_body.contains('$') {
            let mut b = command_body
                .replace('$', "\u{200B}$")
                .replace("\\\u{200B}$", "\\$");
            let mut changed = false;
            for (i, arg) in args.iter::<String>().filter_map(Result::ok).enumerate() {
                changed = true;
                b = b.replace(
                    format!("\u{200B}${}", i).as_str(),
                    &arg.trim_matches('"')
                        .replace('$', "\\$")
                        .replace('@', "@\u{200B}")
                        .replace('\\', "\\\\")
                        .replace('\n', "\\n")
                        .replace('"', "\\\""),
                );
            }

            let argc = args.len() - 1;
            if changed {
                message = serde_json::from_str(&b.replace("\\$", "$"))?;
            }
            changed = false;
            if let Value::Array(a) = &message["default_args"] {
                for (i, arg) in a[argc.min(a.len())..]
                    .iter()
                    .filter_map(Value::as_str)
                    .enumerate()
                {
                    changed = true;
                    println!("Default argument '{}'", arg);
                    b = b.replace(
                        format!("\u{200B}${}", i + argc).as_str(),
                        &arg.replace('$', "\\$"),
                    );
                }
            }
            if changed {
                message = serde_json::from_str(&b.replace("\\$", "$"))?;
            }
        }

        if let Some(b) = message["self_delete"].as_bool() {
            // optionnally overriding the self delete behavior
            delete = b;
        }
        announce(ctx, msg.channel_id, &message).await?;
        if delete {
            msg.delete(ctx).await?;
        }
    } else {
        println!("Could not find custom command \"{}\"", name);
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
pub async fn define(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let name: String = args.single::<String>()?.to_lowercase();

    if RESERVED_NAMES.contains(&name.as_str()) {
        failure!(
            ctx,
            msg,
            "You cannot add a command with the reserved name `{}`",
            name
        );
        return Ok(());
    }

    match get_json_from_message::<Value>(msg).await {
        Ok(mut message) => {
            let mut documentation = message
                .as_object_mut()
                .map(|map| map.remove("documentation").unwrap_or_default())
                .unwrap_or_default();
            if let Some(map) = message["subcommands"].as_object() {
                let s = map
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<&str>>()
                    .join("`, `");
                documentation = Value::String(format!(
                    "{}\n_Subcommands:_  `{}`",
                    documentation.as_str().unwrap_or_default(),
                    s
                ));
            }
            let body = serde_json::to_string_pretty(&message)?;
            println!(
                "adding custom command \"{}\": {}\n({:?})",
                name, body, documentation
            );
            let db_res =
                add_custom_command(ctx, server_id, &name, &body, documentation.as_str()).await;
            if db_res.is_ok()
                && check_command_exists(ctx, server_id, &name)
                    .await
                    .unwrap_or_default()
            {
                success!(ctx, msg);
            } else {
                println!("{:?}", db_res.err());
                failure!(ctx, msg);
            }
        }
        Err(e) => handle_json_error!(ctx, msg, e),
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("remove", "delete")]
async fn custom_command_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let name: String = args.single()?;

    if check_command_exists(ctx, server_id, &name)
        .await
        .unwrap_or_default()
        && remove_custom_command(ctx, server_id, &name).await.is_ok()
        && !check_command_exists(ctx, server_id, &name)
            .await
            .unwrap_or_default()
    {
        success!(ctx, msg);
    } else {
        failure!(ctx, msg);
    }
    Ok(())
}

#[command]
#[aliases("display", "show")]
#[checks(is_admin)]
async fn custom_command_display(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Ok(name) = args.single::<String>() {
        if let Some(command) = get_command_data(ctx, server_id, &name, true).await {
            println!("Displaying command docs...");
            let mut file_too_big = false;
            let bytes = command.body.as_str().as_bytes();
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Custom command: {}", name));
                        if let Some(desc) = &command.description {
                            e.description(desc);
                        }

                        e.field(
                            "Command body",
                            if command.body.len() < 1013 {
                                format!(
                                    "```json\n{}```",
                                    &command.body.replace("```", "`\u{200B}``")
                                )
                            } else {
                                file_too_big = true;
                                "Command body in attachment.".into()
                            },
                            false,
                        );
                        e
                    });
                    if file_too_big {
                        m.add_file((bytes, format!("{}.json", name).as_str()));
                    }
                    m
                })
                .await?;
        } else {
            failure!(ctx, msg, "The custom command does not exist!");
        }
    } else if let Some(list) = get_custom_commands_list(ctx, server_id).await {
        println!("displaying a list of custom commands");
        let mut newline: u32 = 0;
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Custom commands");
                    e.description(
                        list.iter()
                            .map(|(name, desc)| {
                                if desc.is_empty() {
                                    newline += 1;
                                }
                                format!(
                                    "{newline}`{}`{}",
                                    name,
                                    match newline {
                                        0 => format!("  {}\n", desc),
                                        _ => String::new(),
                                    },
                                    newline = match newline {
                                        0 => "",
                                        1 => {
                                            newline += 1;
                                            "\n"
                                        }
                                        _ => ", ",
                                    }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                })
            })
            .await?;
    }
    Ok(())
}
