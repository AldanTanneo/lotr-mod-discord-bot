//! Admin commands, to work with the bot's own permissions system
//!
//! Three categories of people can manage the bot:
//! - Users with the
//! [`MANAGE_BOT_PERMS`][struct@crate::constants::MANAGE_BOT_PERMS]
//! set of permissions, equivalent to
//! `ADMINISTRATOR | MANAGE_CHANNELS | MANAGE_GUILD`;
//! - Users that are promoted to "bot admins" by another admin, using the
//! [`!admin add`][add] command;
//! - And lastly, the bot [owner][OWNER_ID].
//!
//! Most of these commands are only executable by admins, with the exception of
//! the [`!admin`][admin] command which can be used by anyone to display a list
//! of bot admins.
//!
//! # Admin-only commands
//! - [`!prefix`][prefix] displays the current prefix or changes it to the
//! prefix passed in as argument.
//! - [`!admin add`][add] adds a new admin to the database.
//! - [`!admin remove`][remove] removes a bot admin.
//! - [`!blacklist`][blacklist] displays the blacklist, or adds the mentionned
//! channel or users to the blacklist.
//! - [`!announce`][announce] allows bot admin to post messages as the bot,
//! useful for official announcements.
//!
//! # Owner-only commands
//! - [`!floppadmin`][floppadmin] allows the owner to give access to the floppa
//! database.
//! - [`!listguilds`][listguilds] allows the owner to get a list of guilds
//! the bot has been invited in.
//!
//! # About the blacklist
//!
//! Using the [`!blacklist`][blacklist] command, bot admins can add users and

use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;

use crate::check::*;
use crate::constants::{BOT_ID, OWNER_ID};
use crate::database::{
    admin_data::{add_admin, get_admins, remove_admin},
    blacklist::{get_blacklist, update_blacklist},
    config::{get_prefix, set_prefix, PrefixCache},
    floppa::is_floppadmin,
};
use crate::utils::NotInGuild;
use crate::{failure, is_admin, success};

#[command]
#[checks(is_admin)]
#[only_in(guilds)]
#[sub_commands(cache)]
pub async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;
    if args.is_empty() {
        let prefix = get_prefix(ctx, server_id).await;
        msg.reply(
            ctx,
            format!(
                "My prefix here is \"{}\"",
                prefix.unwrap_or_else(|| "!".into())
            ),
        )
        .await?;
    } else {
        let new_prefix = args.single::<String>();
        if let Ok(p) = new_prefix {
            if !p.contains("<@") && set_prefix(ctx, server_id, &p).await.is_ok() {
                success!(ctx, msg, "Set the new prefix to \"{}\"", p);
            } else {
                failure!(ctx, msg, "Failed to set the new prefix!");
            }
        } else {
            failure!(ctx, msg, "Invalid new prefix!");
        }
    }
    Ok(())
}

#[command]
#[owners_only]
#[checks(is_admin)]
async fn cache(ctx: &Context) -> CommandResult {
    let prefix_cache = {
        let data_read = ctx.data.read().await;
        data_read.get::<PrefixCache>().unwrap().clone()
    };
    println!("=== PREFIX CACHE ===\n{:?}\n=== END ===", prefix_cache);
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[sub_commands("add", "remove")]
#[aliases("admins")]
pub async fn admin(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    let admins = get_admins(ctx, server_id).await.unwrap_or_else(Vec::new);

    let mut user_names: Vec<String> = admins.iter().map(|&id| id.mention().to_string()).collect();
    user_names.push(OWNER_ID.mention().to_string());

    let guild_name = server_id.to_partial_guild(ctx).await?.name;
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("List of bot admins");
                e.description(format!("On **{}**\n{}", guild_name, user_names.join("\n")))
            });
            m
        })
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
pub async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(user) = msg.mentions.iter().find(|&user| user.id != BOT_ID) {
        if !(is_admin!(ctx, server_id, user.id) || user.id == OWNER_ID) {
            add_admin(ctx, server_id, user.id, false).await?;
            success!(ctx, msg);
        } else {
            failure!(ctx, msg, "This user is already a bot admin on this server!");
        }
    } else {
        failure!(
            ctx,
            msg,
            "Mention a user you wish to promote to bot admin for this server."
        );
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(user) = msg.mentions.iter().find(|&user| user.id != BOT_ID) {
        if user.id == OWNER_ID {
            failure!(ctx, msg, "You cannot remove this bot admin!");
        } else if is_admin!(ctx, server_id, user.id) {
            remove_admin(ctx, server_id, user.id).await?;
            success!(ctx, msg);
        } else {
            failure!(ctx, msg, "This user is not a bot admin on this server!");
        }
    } else if is_admin!(ctx, server_id, UserId(args.parse().unwrap_or_default())) {
        remove_admin(ctx, server_id, UserId(args.single()?)).await?;
        success!(ctx, msg);
    } else {
        failure!(
            ctx,
            msg,
            "Mention a user you wish to remove from bot admins for this server.",
        );
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
pub async fn blacklist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;
    if args.is_empty() && msg.mentions.is_empty() {
        let (users, channels) = get_blacklist(ctx, server_id).await.unwrap_or_default();

        let mut user_names: Vec<String> = users.iter().map(|&u| u.mention().to_string()).collect();

        let mut channel_names: Vec<String> =
            channels.iter().map(|&c| c.mention().to_string()).collect();

        if user_names.is_empty() {
            user_names.push("None".into());
        }
        if channel_names.is_empty() {
            channel_names.push("None".into());
        }

        let guild_name = server_id.to_partial_guild(ctx).await?.name;
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Blacklist");
                    e.description(format!("On **{}**", guild_name));
                    e.field("Blacklisted users:", user_names.join("\n"), true);
                    e.field("Blacklisted channels:", channel_names.join("\n"), true)
                });
                m
            })
            .await?;
    } else {
        update_blacklist(ctx, msg, args).await?;
    }
    Ok(())
}

#[command]
#[owners_only]
pub async fn floppadmin(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(user) = msg
        .mentions
        .iter()
        .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
    {
        if is_floppadmin(ctx, server_id, user.id)
            .await
            .unwrap_or_default()
        {
            add_admin(ctx, server_id, user.id, false)
        } else {
            add_admin(ctx, server_id, user.id, true)
        }
        .await?;
        success!(ctx, msg);
    } else {
        failure!(
            ctx,
            msg,
            "Mention a user you wish to promote to floppadmin for this server.",
        );
    }
    Ok(())
}

#[command]
#[only_in(dms)]
#[owners_only]
#[aliases("guilds")]
pub async fn listguilds(ctx: &Context) -> CommandResult {
    let mut id = GuildId(0);
    let owner = OWNER_ID.to_user(&ctx).await?;
    let mut first = true;
    let mut count = 0;
    while let Ok(vec) = ctx
        .http
        .get_guilds(Some(&serenity::http::GuildPagination::After(id)), Some(20))
        .await
    {
        if vec.is_empty() {
            break;
        } else {
            let guild_names = vec
                .iter()
                .map(|g| format!("{} (`{}`)", g.name, g.id))
                .collect::<Vec<_>>()
                .join("\n");
            count += vec.len();
            id = vec[vec.len() - 1].id;
            if first {
                first = false;
                owner
                    .dm(&ctx, |m| m.content(format!("**Guilds:**\n{}", guild_names)))
                    .await?;
            } else {
                owner.dm(&ctx, |m| m.content(guild_names)).await?;
            }
        }
    }
    owner
        .dm(ctx, |m| m.content(format!("*{} guilds*", count)))
        .await?;
    Ok(())
}
