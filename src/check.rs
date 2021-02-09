use crate::{LOTR_DISCORD, OWNER_ID};
use serenity::framework::standard::{
    macros::{check, hook},
    DispatchError, Reason,
};
use serenity::futures::future::join;
use serenity::model::{
    channel::Message, id::GuildId, prelude::ReactionType, user::User, Permissions,
};
use serenity::prelude::*;

use crate::database::{
    admin_data, blacklist::check_blacklist, config::get_minecraft_ip, Blacklist,
};

pub async fn bot_admin(ctx: &Context, msg: &Message) -> bool {
    admin_data::is_admin(ctx, msg.guild_id, msg.author.id)
        .await
        .is_some()
}

pub async fn has_permission(
    ctx: &Context,
    guild: Option<GuildId>,
    user: &User,
    perm: Permissions,
) -> bool {
    if let Some(guild) = guild {
        if let Ok(g) = guild.to_partial_guild(&ctx).await {
            for (role_id, role) in g.roles.iter() {
                if role.permissions.intersects(perm)
                    && user.has_role(ctx, guild, role_id).await.unwrap_or(false)
                {
                    return true;
                }
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
        && !has_permission(
            ctx,
            msg.guild_id,
            &msg.author,
            Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
        )
        .await
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
pub async fn is_lotr_discord(_: &Context, msg: &Message) -> Result<(), Reason> {
    if msg.guild_id != Some(LOTR_DISCORD) {
        Err(Reason::Log(
            "Tried to use !tos on another discord".to_string(),
        ))
    } else {
        Ok(())
    }
}

#[check]
pub async fn is_admin(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if msg.author.id == OWNER_ID
        || bot_admin(ctx, msg).await
        || has_permission(
            ctx,
            msg.guild_id,
            &msg.author,
            Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
        )
        .await
    {
        Ok(())
    } else {
        Err(Reason::User(
            "You are not an admin on this server!".to_string(),
        ))
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
                    println!("Error sending blacklist warning")
                }
            }
            Reason::Log(err_message) => {
                println!("Check failed: {}", err_message)
            }
            _ => (),
        }
    }
}

#[check]
pub async fn is_minecraft_server(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    if get_minecraft_ip(ctx, msg.guild_id).await.is_some() {
        Ok(())
    } else {
        if bot_admin(ctx, msg).await
            || msg.author.id == OWNER_ID
            || has_permission(
                ctx,
                msg.guild_id,
                &msg.author,
                Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
            )
            .await
        {
            println!("Bypassed minecraft server check");
            Ok(())
        } else {
            Err(Reason::Log("Not a minecraft server".to_string()))
        }
    }
}
