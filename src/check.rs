//! Checks that are executed before running a command.
//!
//! They can then decide wether to execute the command, ignore it
//! or send a warning to the user.
//!
//! [`allowed_blacklist`] checks wether the command is allowed through
//! the [blacklist][crate::database::blacklist]. It fails if the channel or
//! the user is blacklisted.
//! Bot admins can bypass this check.
//!
//! [`is_admin`] checks wether the user is either the owner, a bot admin,
//! or has the [`struct@MANAGE_BOT_PERMS`] permissions.
//!
//! [`is_minecraft_server`] checks wether there is a server IP registered
//! with the guild. It fails if there is none, but is bypassed by bot
//! admins.
//!
//! The [`dispatch_error_hook`] deals with the checks that fail and warns
//! the user and/or log the error accordingly.
//!
//! The [`after_hook`] logs any command error to the bot console.

use serenity::framework::standard::{
    macros::{check, hook},
    CommandError, DispatchError, Reason,
};
use serenity::futures::future::join;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::constants::{LOTR_DISCORD, MANAGE_BOT_PERMS, OWNER_ID};
use crate::database::{blacklist::check_blacklist, config::get_minecraft_ip};
use crate::is_admin;
use crate::utils::has_permission;

#[check]
#[name = "allowed_blacklist"]
pub async fn allowed_blacklist(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let server_id = msg
        .guild_id
        .ok_or_else(|| Reason::Log("Not in a guild".into()))?;

    let thread_parent_channel = msg
        .channel_id
        .to_channel(ctx)
        .await
        .map_err(|e| Reason::Log(format!("Could not retrieve channel: {}", e)))?
        .guild()
        .map(|g| g.thread_metadata.map(|_| g.category_id).flatten())
        .ok_or_else(|| Reason::Log("Not in a guild".into()))?;

    let mut channel_id = msg.channel_id;

    if let Some(parent_channel) = thread_parent_channel {
        channel_id = parent_channel;
    }

    if check_blacklist(ctx, server_id, msg.author.id, channel_id)
        .await
        .unwrap_or(true)
        && !is_admin!(ctx, msg)
        && msg.author.id != OWNER_ID
        && !has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await
    {
        msg.delete(ctx)
            .await
            .map_err(|_| Reason::Log("Blacklisted".into()))?;
        Err(Reason::UserAndLog {
            user: "You are not allowed to use this command here.".into(),
            log: "Sending DM warning".into(),
        })
    } else {
        Ok(())
    }
}

#[check]
#[name = "is_admin"]
pub async fn is_admin(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let server_id = msg.guild_id.unwrap_or_default();
    if msg.author.id == OWNER_ID
        || is_admin!(ctx, msg)
        || has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await
    {
        Ok(())
    } else {
        Err(Reason::User("You are not an admin on this server!".into()))
    }
}

#[check]
#[name = "is_minecraft_server"]
pub async fn is_minecraft_server(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let server_id = msg
        .guild_id
        .ok_or_else(|| Reason::Log("Not in a guild".into()))?;
    if get_minecraft_ip(ctx, server_id).await.is_some() {
        Ok(())
    } else if is_admin!(ctx, msg)
        || msg.author.id == OWNER_ID
        || has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await
    {
        println!("Bypassed minecraft server check");
        Ok(())
    } else {
        Err(Reason::Log("Not a minecraft server".into()))
    }
}

#[check]
#[name = "is_lotr_discord"]
pub async fn is_lotr_discord(_: &Context, msg: &Message) -> Result<(), Reason> {
    if msg.guild_id == Some(LOTR_DISCORD) || msg.author.id == OWNER_ID {
        Ok(())
    } else {
        Err(Reason::Log(
            "Tried to use the bug tracker outside of LOTR Discord".into(),
        ))
    }
}

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(s, reason) => {
            println!("Check failed: {}", s);
            match reason {
                Reason::User(err_message) => {
                    match join(
                        msg.reply(ctx, err_message),
                        msg.react(ctx, ReactionType::from('❌')),
                    )
                    .await
                    {
                        (Err(_), _) | (_, Err(_)) => println!("Error sending failure message"),
                        _ => (),
                    }
                }
                Reason::UserAndLog { user, log: _ } => {
                    if msg.author.dm(ctx, |m| m.content(user)).await.is_err() {
                        println!("Error sending blacklist warning")
                    }
                }
                Reason::Log(err_message) => {
                    println!("{}", err_message);
                }
                _ => println!("(Unknown reason)"),
            }
        }
        DispatchError::OnlyForGuilds => {
            if let Err(e) = msg
                .reply(ctx, "This command cannot be executed in DMs!")
                .await
            {
                println!("Error sending guild-only warning: {:?}", e);
            }
        }
        DispatchError::Ratelimited(rate_limit_info) => {
            if rate_limit_info.is_first_try {
                if let Err(e) = msg
                    .reply(ctx, "Wait a few seconds before using this command again!")
                    .await
                {
                    println!("Error sending ratelimited warning: {:?}", e)
                }
            }
        }
        _ => println!("Dispatch error: {:?}", error),
    }
}

#[hook]
pub async fn after_hook(
    _: &Context,
    msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    if let Err(why) = error {
        println!(
            "=== ERROR REPORT ===
Error in command `{}`: {:?}
=== MESSAGE ===
Author: {} ({})
Guild: {}
Channel: {}
Content: {}",
            cmd_name,
            why,
            msg.author.tag(),
            msg.author.id,
            msg.guild_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "None".into()),
            msg.channel_id,
            msg.content,
        );
    }
}
