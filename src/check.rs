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

use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID};
use crate::database::{blacklist::check_blacklist, config::get_minecraft_ip, Blacklist};
use crate::is_admin;
use crate::utils::has_permission;

#[check]
pub async fn allowed_blacklist(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if check_blacklist(ctx, msg, false)
        .await
        .unwrap_or(Blacklist::IsBlacklisted(true))
        .is_blacklisted()
        && !is_admin!(ctx, msg)
        && msg.author.id != OWNER_ID
        && !has_permission(ctx, msg.guild_id, msg.author.id, *MANAGE_BOT_PERMS).await
    {
        msg.delete(ctx)
            .await
            .map_err(|_| Reason::Log("Blacklisted".to_string()))?;
        Err(Reason::UserAndLog {
            user: "You are not allowed to use this command here.".to_string(),
            log: "Sending DM warning".to_string(),
        })
    } else {
        Ok(())
    }
}

#[check]
pub async fn is_admin(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if msg.author.id == OWNER_ID
        || is_admin!(ctx, msg)
        || has_permission(ctx, msg.guild_id, msg.author.id, *MANAGE_BOT_PERMS).await
    {
        Ok(())
    } else {
        Err(Reason::User(
            "You are not an admin on this server!".to_string(),
        ))
    }
}

#[check]
pub async fn is_minecraft_server(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if get_minecraft_ip(ctx, msg.guild_id).await.is_some() {
        Ok(())
    } else if is_admin!(ctx, msg)
        || msg.author.id == OWNER_ID
        || has_permission(ctx, msg.guild_id, msg.author.id, *MANAGE_BOT_PERMS).await
    {
        println!("Bypassed minecraft server check");
        Ok(())
    } else {
        Err(Reason::Log("Not a minecraft server".to_string()))
    }
}

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::CheckFailed(s, reason) = error {
        println!("{}", s);
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
                };
            }
            Reason::UserAndLog { user, log: _ } => {
                if msg.author.dm(ctx, |m| m.content(user)).await.is_err() {
                    println!("Error sending blacklist warning");
                }
            }
            Reason::Log(err_message) => {
                println!("Check failed: {}", err_message);
            }
            _ => (),
        }
    }
}

#[hook]
pub async fn after_hook(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        println!("Error in `{}`: {:?}", cmd_name, why);
    }
}
