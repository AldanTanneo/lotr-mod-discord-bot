use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::framework::Framework;
use serenity::futures::future::join;
use serenity::model::channel::Message;
use serenity::prelude::Mentionable;

use crate::announcement::{announce, Announcement};
use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID, RESERVED_NAMES};
use crate::database::{
    blacklist::check_blacklist,
    custom_commands::{
        add_custom_command, check_command_exists, get_command_data, get_custom_commands_list,
        remove_custom_command,
    },
};
use crate::utils::{get_json_from_message, has_permission, to_json_safe_string, NotInGuild};
use crate::{check::*, FrameworkKey};
use crate::{failure, handle_json_error, is_admin, success};

/// Simple rolling hash function to compare strings
pub const fn hash_string(string: &str) -> u64 {
    // prime number slightly under 2^56
    const MOD: u64 = 3u64.pow(35) - 28;
    // prime number slightly under 2^8
    const POW: u64 = 251;

    let value = string.as_bytes();
    let mut hash = 0;
    let mut i = 0;
    let n = value.len();
    while i < n {
        hash = (hash * POW) % MOD;
        hash = (hash + value[i] as u64) % MOD;
        i += 1;
    }
    hash
}

pub async fn manual_dispatch(
    ctx: Context,
    mimicked_message: &Message,
    dispatched_command: &str,
) -> CommandResult {
    let framework = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<FrameworkKey>()
            .expect("There should be a framework in the typemap")
            .clone()
    };

    let prefix =
        crate::database::config::get_prefix(&ctx, mimicked_message.guild_id.unwrap_or_default())
            .await
            .unwrap_or_else(|| "!".to_string());

    let mut custom_message = mimicked_message.clone();
    custom_message.content = prefix + dispatched_command;

    let previous_msg_hash = Value::Number(hash_string(&mimicked_message.content).into());
    if let Some(vec) = custom_message.nonce.as_array_mut() {
        if vec.contains(&previous_msg_hash) {
            println!("=== ABORT: POSSIBLE LOOP ===");
            return Ok(());
        }
        vec.push(previous_msg_hash);
    } else {
        custom_message.nonce = Value::Array(vec![previous_msg_hash]);
    }

    println!("Manual dispatch of content: {}", custom_message.content);
    tokio::task::spawn(async move { framework.dispatch(ctx, custom_message).await });

    Ok(())
}

#[command]
#[aliases("command")]
#[sub_commands(define, custom_command_remove, custom_command_display)]
pub async fn custom_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let Some(server_id) = msg.guild_id else {
        // No custom commands for DMs!
        return Ok(());
    };

    let name = args.single::<String>()?.to_lowercase(); // getting command name
    let subcommand = args.current(); // getting possible subcommand but not advancing

    if let Some(command_data) = get_command_data(ctx, server_id, &name, false).await {
        println!("Custom command execution: {}", msg.content);

        let mut message: Announcement =
            serde_json::from_str(&command_data.body.replace("\\$", "$"))?;
        let mut delete = message.extra["self_delete"].as_bool().unwrap_or_default();

        let default_command_type = message.extra["type"].as_str();
        let subcommands_object = &message.extra["subcommands"];
        // early interrupt in case of blacklist / admin command
        let command_type = if let Some(subcommand) = subcommand {
            // optionnally overriding the command type
            if subcommands_object[subcommand]["type"].is_string() {
                subcommands_object[subcommand]["type"].as_str()
            } else if let Some(subcommand_alias) = subcommands_object[subcommand].as_str() {
                if subcommands_object[subcommand_alias]["type"].is_string() {
                    subcommands_object[subcommand_alias]["type"].as_str()
                } else {
                    default_command_type
                }
            } else {
                default_command_type
            }
        } else {
            default_command_type
        };

        let is_alias = command_type == Some("alias");

        if let Some(s) = command_type {
            if s == "group" {
                return Ok(());
            }
            let is_admin = msg.author.id == OWNER_ID
                || is_admin!(ctx, msg)
                || has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await;
            if !is_admin {
                if s == "meme"
                    && check_blacklist(ctx, server_id, msg.author.id, msg.channel_id)
                        .await
                        .unwrap_or(true)
                {
                    println!(
                        "=== BLACKLIST ===\nUser: {} {:?}\nGuild: {}
Channel: {:?}\nMessage: {}\n=== END ===",
                        msg.author.tag(),
                        msg.author.id,
                        msg.guild_id
                            .map_or_else(|| "None".into(), |id| format!("{id:?}")),
                        msg.channel_id,
                        msg.content
                    );
                    return match join(
                        msg.author.dm(ctx, |m| {
                            m.embed(|e| {
                                e.colour(serenity::utils::colours::branding::RED)
                                    .description("You are not allowed to use this command here.")
                            })
                        }),
                        msg.delete(ctx),
                    )
                    .await
                    {
                        (Err(e), _) | (_, Err(e)) => Err(CommandError::from(e)),
                        _ => Ok(()),
                    };
                } else if s == "admin" {
                    failure!(ctx, msg, "You are not an admin on this server!");
                    return Ok(());
                }
            }
        }

        let mut command_body = command_data.body;
        if let Some(subcommand) = subcommand {
            if subcommands_object[subcommand].is_object() {
                command_body = serde_json::to_string(&subcommands_object[subcommand])?;
                message = serde_json::from_str(&command_body)?;
                args.advance();
            } else if let Some(subcommand_alias) = subcommands_object[subcommand].as_str() {
                if subcommands_object[subcommand_alias].is_object() {
                    command_body = serde_json::to_string(&subcommands_object[subcommand_alias])?;
                    message = serde_json::from_str(&command_body)?;
                    args.advance();
                }
            }
        }

        if command_body.contains('$') {
            let mut changed = false;

            let mut b = command_body
                .replace('$', "\u{200B}$")
                .replace("\\\u{200B}$", "\\$");

            if b.contains("\u{200B}$me")
                || b.contains("\u{200B}$ping")
                || b.contains("\u{200B}$channel")
            {
                changed = true;
                b = b
                    .replace("\u{200B}$me", &to_json_safe_string(&msg.author.name))
                    .replace("\u{200B}$ping", &msg.author.mention().to_string())
                    .replace("\u{200B}$channel", &msg.channel_id.mention().to_string());
            }

            if b.contains("\u{200B}$as_url") {
                changed = true;
                b = b.replace("\u{200B}$as_url", &urlencoding::encode(args.rest()));
            }

            if b.contains("\u{200B}$args") {
                changed = true;
                b = b.replace("\u{200B}$args", &to_json_safe_string(args.rest()));
            } else {
                args.iter::<String>()
                    .filter_map(Result::ok)
                    .enumerate()
                    .for_each(|(i, arg)| {
                        let key = format!("\u{200B}${i}");
                        if b.contains(&key) {
                            changed = true;
                            b = b.replace(
                                key.as_str(),
                                &to_json_safe_string(
                                    arg.replace('$', "\\$")
                                        .replace('@', "@\u{200B}")
                                        .trim_matches('"'),
                                ),
                            );
                        }
                    });
            }

            let argc = args.len() - 1;
            if changed {
                message = serde_json::from_str(&b.replace("\\$", "$"))?;
            }
            changed = false;
            if let Value::Array(a) = &message.extra["default_args"] {
                for (i, arg) in a[argc.min(a.len())..]
                    .iter()
                    .filter_map(Value::as_str)
                    .enumerate()
                {
                    changed = true;
                    println!("Default argument '{arg}'");
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

        if let Some(b) = message.extra["self_delete"].as_bool() {
            // optionally overriding the self delete behavior
            delete = b;
        }

        if is_alias {
            if let Some(command) = message.extra["command"].as_str() {
                manual_dispatch(ctx.clone(), msg, command).await?;
                return Ok(());
            }
        }
        announce(ctx, msg.channel_id, &message).await?;
        if delete {
            msg.delete(ctx).await?;
        }
    } else if msg.nonce.is_array() {
        failure!(
            ctx,
            msg,
            "This command is an alias of the deleted command `{}`",
            name
        );
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[only_in(guilds)]
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
            if message["type"].as_str() == Some("alias") && !message["command"].is_string() {
                failure!(
                    ctx,
                    msg,
                    "Custom commands with the `\"alias\"` type require a `\"command\"` string field."
                );
                return Ok(());
            }

            let mut documentation = message
                .as_object_mut()
                .map(|map| map.remove("documentation").unwrap_or_default())
                .unwrap_or_default();
            if let Some(map) = message["subcommands"].as_object() {
                // validate that all subcommands are well defined
                if let Some((key, val)) = map.iter().find_map(|(key, val)| {
                    val.as_str().and_then(|v| {
                        if !(map.contains_key(v) && map[v].is_object()) || v == key {
                            Some((key, v))
                        } else {
                            None
                        }
                    })
                }) {
                    failure!(ctx, msg, "The alias `{:?}: {:?}` is not defined!", key, val);
                    return Ok(());
                }
                // validate that all aliases subcommands have a "command" field
                if let Some((key, _val)) = map.iter().find(|(_key, val)| {
                    val["type"].as_str() == Some("alias") && !val["command"].is_string()
                }) {
                    failure!(
                        ctx,
                        msg,
                        "The subcommand `{:?}` with the `\"alias\"` type requires a `\"command\"` string field.",
                        key
                    );
                    return Ok(());
                }

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
                "adding custom command \"{name}\": {body}\n({documentation:?})"
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
async fn custom_command_display(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(name) = args.current() {
        if let Some(command) = get_command_data(ctx, server_id, name, true).await {
            println!("Displaying command docs for command {}", command.name);
            let mut file_too_big = false;
            let bytes = command.body.as_str().as_bytes();
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Custom command: {name}"));
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
                        m.add_file((bytes, format!("{name}.json").as_str()));
                    }
                    m
                })
                .await?;
        } else {
            failure!(ctx, msg, "The custom command does not exist!");
        }
    } else if let Some(list) = get_custom_commands_list(ctx, server_id).await {
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
                                        0 => format!("  {desc}\n"),
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
                            .collect::<String>(),
                    )
                })
            })
            .await?;
    }
    Ok(())
}
