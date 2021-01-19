use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::{
    channel::Message,
    id::{ChannelId, GuildId},
    prelude::ReactionType,
};
use serenity::prelude::*;

use crate::announcement;
use crate::check::{ALLOWED_BLACKLIST_CHECK, IS_ADMIN_CHECK};
use crate::constants::{BOT_ID, LOTR_DISCORD, OWNER_ID};
use crate::database::{
    admin_data::{add_admin, get_admins, is_admin, remove_admin},
    blacklist::{check_blacklist, update_blacklist},
    config::{get_prefix, set_prefix},
    floppa::is_floppadmin,
    Blacklist::IsBlacklisted,
};

#[command]
#[checks(is_admin)]
#[only_in(guilds)]
#[max_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let prefix = get_prefix(ctx, msg.guild_id).await;
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
            if !p.contains("<@") && set_prefix(ctx, msg.guild_id, &p, true).await.is_ok() {
                msg.reply(ctx, format!("Set the new prefix to \"{}\"", p))
                    .await?;
            } else {
                msg.reply(ctx, "Failed to set the new prefix!").await?;
            }
        } else {
            msg.reply(ctx, "Invalid new prefix!").await?;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[max_args(1)]
#[min_args(1)]
async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(user) = msg
        .mentions
        .iter()
        .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
    {
        if !is_admin(ctx, msg.guild_id, user.id).await.is_some() {
            add_admin(ctx, msg.guild_id, user.id, false, false).await?;
            msg.react(ctx, ReactionType::from('✅')).await?;
        } else {
            msg.reply(ctx, "This user is already a bot admin on this server!")
                .await?;
        }
    } else {
        msg.reply(
            ctx,
            "Mention a user you wish to promote to bot admin for this server.",
        )
        .await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[max_args(1)]
#[min_args(1)]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(user) = msg
        .mentions
        .iter()
        .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
    {
        if is_admin(ctx, msg.guild_id, user.id).await.is_some() {
            remove_admin(ctx, msg.guild_id, user.id).await?;
            msg.react(ctx, ReactionType::from('✅')).await?;
        } else {
            msg.reply(ctx, "This user is not a bot admin on this server!")
                .await?;
        }
    } else {
        msg.reply(
            ctx,
            "Mention a user you wish to remove from bot admins for this server.",
        )
        .await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_else(Vec::new);

    let mut user_names: Vec<String> = admins.iter().map(|&id| id.mention().to_string()).collect();
    user_names.push(OWNER_ID.mention().to_string());

    let guild_name = msg
        .guild_id
        .unwrap_or(LOTR_DISCORD)
        .to_partial_guild(ctx)
        .await?
        .name;
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
#[checks(is_admin, allowed_blacklist)]
async fn blacklist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() && msg.mentions.is_empty() {
        let (users, channels) = check_blacklist(ctx, msg, true)
            .await
            .unwrap_or(IsBlacklisted(true))
            .get_list();

        let mut user_names: Vec<String> = users.iter().map(|&u| u.mention().to_string()).collect();

        let mut channel_names: Vec<String> =
            channels.iter().map(|&c| c.mention().to_string()).collect();

        if user_names.is_empty() {
            user_names.push("None".into());
        }
        if channel_names.is_empty() {
            channel_names.push("None".into());
        }

        let guild_name = msg
            .guild_id
            .unwrap_or(LOTR_DISCORD)
            .to_partial_guild(ctx)
            .await?
            .name;
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
#[only_in(guilds)]
#[checks(is_admin)]
async fn announce(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = serenity::utils::parse_channel(args.single::<String>()?.trim());
    if let Some(id) = channel {
        if msg.guild_id
            != ChannelId(id)
                .to_channel(ctx)
                .await?
                .guild()
                .map(|c| c.guild_id)
        {
            msg.reply(
                ctx,
                "You can only announce in the same server as the one you are in!",
            )
            .await?;
            msg.react(ctx, ReactionType::from('❌')).await?;
            return Ok(());
        };
        let message = if msg.attachments.is_empty() {
            let content = &msg.content;
            let (a, b) = (
                content.find('{').unwrap_or(0),
                content.rfind('}').unwrap_or(0),
            );
            serde_json::from_str(&content[a..=b])
        } else {
            let a = &msg.attachments[0];
            if a.size <= 51200 {
                let json_data = a.download().await?;
                serde_json::from_slice(&json_data)
            } else {
                msg.reply(ctx, "Attachment is too big! Filesize must be under 50KB.")
                    .await?;
                return Ok(());
            }
        };
        if message.is_ok()
            && announcement::announce(ctx, ChannelId(id), message.unwrap())
                .await
                .is_ok()
        {
            msg.react(ctx, ReactionType::from('✅')).await?;
        } else {
            msg.reply(
                ctx,
                "Error sending the message! Check your json content and/or the bot permissions.",
            )
            .await?;
            msg.react(ctx, ReactionType::from('❌')).await?;
        };
    } else {
        msg.reply(ctx, "The first argument must be a channel mention!")
            .await?;
        msg.react(ctx, ReactionType::from('❌')).await?;
    }
    Ok(())
}

#[command]
#[owners_only]
async fn floppadmin(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(user) = msg
        .mentions
        .iter()
        .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
    {
        if !is_floppadmin(ctx, msg.guild_id, user.id)
            .await
            .unwrap_or(false)
        {
            if !is_admin(ctx, msg.guild_id, user.id).await.is_some() {
                add_admin(ctx, msg.guild_id, user.id, false, true)
            } else {
                add_admin(ctx, msg.guild_id, user.id, true, true)
            }
        } else {
            add_admin(ctx, msg.guild_id, user.id, true, false)
        }
        .await?;
        msg.react(ctx, ReactionType::from('✅')).await?;
    } else {
        msg.reply(
            ctx,
            "Mention a user you wish to promote to floppadmin for this server.",
        )
        .await?;
    }
    Ok(())
}

#[command]
#[only_in(dms)]
#[owners_only]
#[aliases("guilds")]
async fn listguilds(ctx: &Context) -> CommandResult {
    let mut id = GuildId(0);
    let owner = OWNER_ID.to_user(&ctx).await?;
    let mut first = true;
    let mut count = 0;
    while let Ok(vec) = ctx
        .http
        .get_guilds(&serenity::http::GuildPagination::After(id), 20)
        .await
    {
        if vec.is_empty() {
            break;
        } else {
            let guild_names = vec
                .iter()
                .map(|g| g.name.clone())
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
