use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::{channel::Message, prelude::ReactionType, Permissions};

use crate::announcement::announce;
use crate::check::{bot_admin, has_permission, IS_ADMIN_CHECK};
use crate::constants::{MAX_JSON_FILE_SIZE, OWNER_ID};
use crate::database::{
    blacklist::check_blacklist,
    custom_commands::{
        add_custom_command, check_command_exists, get_command_data, get_custom_commands_list,
        remove_custom_command,
    },
    Blacklist,
};

#[command]
#[only_in(guilds)]
#[aliases("command")]
#[sub_commands(define, custom_command_remove, custom_command_display)]
pub async fn custom_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Custom command execution...");
    let name = args.single::<String>()?.to_lowercase();
    if let Some(command_data) = get_command_data(ctx, msg.guild_id, &name, false).await {
        let mut body = if command_data.body.contains('$') {
            command_data
                .body
                .replace('$', "\u{200B}$")
                .replace("\\\u{200B}$", "\\$")
        } else {
            command_data.body
        };
        for (i, arg) in args.iter::<String>().enumerate() {
            body = body.replace(
                format!("\u{200B}${}", i).as_str(),
                &arg?
                    .as_str()
                    .trim_matches('"')
                    .replace('$', "\\$")
                    .replace('@', "@\u{200B}")
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('"', "\\\""),
            );
        }
        println!("{}", body);
        let mut message: Value = serde_json::from_str(&body.replace("\\$", "$"))?;
        if let Value::Array(a) = &message["default_args"] {
            let argc = args.len() - 1;
            let changed = body.contains(format!("\u{200B}${}", argc).as_str());
            for (i, arg) in a[argc.min(a.len())..].iter().enumerate() {
                if let Value::String(s) = arg {
                    println!("Default argument '{}'", s);
                    body = body.replace(
                        format!("\u{200B}${}", i + argc).as_str(),
                        &s.replace('$', "\\$"),
                    );
                }
            }
            if changed {
                message = serde_json::from_str(&body.replace("\\$", "$"))?;
            }
        }
        if let Value::String(s) = &message["type"] {
            if s == "meme"
                && msg.author.id != OWNER_ID
                && check_blacklist(ctx, msg, false)
                    .await
                    .unwrap_or(Blacklist::IsBlacklisted(true))
                    .is_blacklisted()
                && !bot_admin(ctx, msg).await
                && !has_permission(
                    ctx,
                    msg.guild_id,
                    &msg.author,
                    Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
                )
                .await
            {
                msg.delete(ctx).await?;
                msg.author
                    .dm(ctx, |m| {
                        m.content("You are not allowed to use this command here!")
                    })
                    .await?;
                return Ok(());
            } else if s == "admin"
                && msg.author.id != OWNER_ID
                && !bot_admin(ctx, msg).await
                && !has_permission(
                    ctx,
                    msg.guild_id,
                    &msg.author,
                    Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
                )
                .await
            {
                msg.react(ctx, ReactionType::from('❌')).await?;
                msg.reply(ctx, "You are not an admin on this server!")
                    .await?;
                return Ok(());
            }
        };
        announce(ctx, msg.channel_id, message).await?;
    } else {
        println!("Could not find custom command \"{}\"", name);
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
pub async fn define(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name: String = args.single()?;
    let update = check_command_exists(ctx, msg.guild_id, &name)
        .await
        .unwrap_or(false);
    let message = if msg.attachments.is_empty() {
        let content = &msg.content;
        let (a, b) = (
            content.find('{').unwrap_or(0),
            content.rfind('}').unwrap_or(0),
        );
        serde_json::from_str::<Value>(&content[a..=b])
    } else {
        let a = &msg.attachments[0];
        if a.size <= MAX_JSON_FILE_SIZE {
            let json_data = a.download().await?;
            serde_json::from_slice::<Value>(&json_data)
        } else {
            msg.react(ctx, ReactionType::from('❌')).await?;
            msg.reply(ctx, "Attachment is too big! Filesize must be under 25KB.")
                .await?;
            return Ok(());
        }
    };

    if let Ok(mut message) = message {
        let documentation = message
            .as_object_mut()
            .map(|map| map.remove("documentation").unwrap_or_default())
            .unwrap_or_default();
        let body = serde_json::to_string_pretty(&message)?;
        println!(
            "adding custom command \"{}\": {}\n({:?})",
            name, body, documentation
        );
        let db_res = add_custom_command(
            ctx,
            msg.guild_id,
            &name,
            body,
            documentation.as_str(),
            update,
        )
        .await;
        if db_res.is_ok()
            && check_command_exists(ctx, msg.guild_id, &name)
                .await
                .unwrap_or(false)
        {
            msg.react(ctx, ReactionType::from('✅'))
        } else {
            println!("{:?}", db_res.err());
            msg.react(ctx, ReactionType::from('❌'))
        }
        .await?;
    } else {
        msg.reply(ctx, "Error defining the command! Check your JSON content.")
            .await?;
        msg.react(ctx, ReactionType::from('❌')).await?;
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("remove", "delete")]
pub async fn custom_command_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name: String = args.single()?;
    if check_command_exists(ctx, msg.guild_id, &name)
        .await
        .unwrap_or(false)
        && remove_custom_command(ctx, msg.guild_id, &name)
            .await
            .is_ok()
        && !check_command_exists(ctx, msg.guild_id, &name)
            .await
            .unwrap_or(false)
    {
        msg.react(ctx, ReactionType::from('✅'))
    } else {
        msg.react(ctx, ReactionType::from('❌'))
    }
    .await?;
    Ok(())
}

#[command]
#[aliases("display", "show")]
#[checks(is_admin)]
pub async fn custom_command_display(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(name) = args.single::<String>() {
        if let Some(command) = get_command_data(ctx, msg.guild_id, &name, true).await {
            println!("Displaying command docs...");
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Custom command: {}", name));
                        if let Some(desc) = command.description {
                            e.description(desc);
                        }

                        e.field(
                            "Command body",
                            if command.body.len() < 1013 {
                                format!("```json\n{}```", command.body)
                            } else {
                                "_Too long to display here_".into()
                            },
                            false,
                        )
                    })
                })
                .await?;
        } else {
            msg.reply(ctx, "The custom command does not exist!").await?;
            msg.react(ctx, ReactionType::from('❌')).await?;
        }
    } else if let Some(list) = get_custom_commands_list(ctx, msg.guild_id).await {
        println!("displaying a list of custom commands");
        let mut newline = 0;
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Custom commands");
                    e.description(
                        list.iter()
                            .map(|(name, desc)| {
                                format!(
                                    "{skip}`{}`  {}\n",
                                    name,
                                    if desc.is_empty() {
                                        newline += 1;
                                        "_No description_"
                                    } else {
                                        desc
                                    },
                                    skip = if newline == 1 {
                                        newline += 1;
                                        "\n"
                                    } else {
                                        ""
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
