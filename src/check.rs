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

use serenity::framework::standard::{
    macros::{check, hook},
    DispatchError, Reason,
};
use serenity::futures::future::join;
use serenity::model::{
    channel::Message, id::GuildId, prelude::ReactionType, user::User, Permissions,
};
use serenity::prelude::*;

use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID};
use crate::database::{
    admin_data, blacklist::check_blacklist, config::get_minecraft_ip, Blacklist,
};

/// Simple wrapper function for [`database::admin_data::is_admin`][crate::database::admin_data::is_admin].
///
/// Returns `true` if the message author is a bot admin in the current guild,
/// `false` otherwise (or if the database query failed).
#[doc(inline)]
pub async fn bot_admin(ctx: &Context, msg: &Message) -> bool {
    admin_data::is_admin(ctx, msg.guild_id, msg.author.id)
        .await
        .is_some()
}

/// Checks a [`User`]'s permissions.
///
/// Returns `true` if `user` has the [permissions][Permissions] `perm` in the
/// `guild`. If `guild` is [`None`], or if the user lacks the permissions,
/// returns `false`.
pub async fn has_permission(
    ctx: &Context,
    guild: Option<GuildId>,
    user: &User,
    perm: Permissions,
) -> bool {
    if let Some(guild) = guild {
        if let Ok(g) = guild.to_partial_guild(&ctx).await {
            if let Ok(m) = g.member(ctx, user.id).await {
                return m
                    .permissions(ctx)
                    .await
                    .unwrap_or_default()
                    .intersects(perm);
            }
        }
    }
    false
}

#[check]
pub async fn allowed_blacklist(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if check_blacklist(ctx, msg, false)
        .await
        .unwrap_or(Blacklist::IsBlacklisted(true))
        .is_blacklisted()
        && !bot_admin(ctx, msg).await
        && msg.author.id != OWNER_ID
        && !has_permission(ctx, msg.guild_id, &msg.author, *MANAGE_BOT_PERMS).await
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
        || bot_admin(ctx, msg).await
        || has_permission(ctx, msg.guild_id, &msg.author, *MANAGE_BOT_PERMS).await
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
    } else if bot_admin(ctx, msg).await
        || msg.author.id == OWNER_ID
        || has_permission(ctx, msg.guild_id, &msg.author, *MANAGE_BOT_PERMS).await
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
