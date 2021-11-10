use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::futures::future::join3;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::check::*;
use crate::database::qa_data;

#[command]
#[only_in(guilds)]
#[checks(is_admin, is_lotr_discord)]
#[aliases("moderator")]
pub async fn qa_moderator(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.expect("Should be only used in guilds");

    if msg.mentions.is_empty() {
        crate::failure!(ctx, msg, "The first argument should be a user mention!");
    }

    for user in &msg.mentions {
        if qa_data::is_qa_moderator(ctx, user.id, guild_id)
            .await
            .unwrap_or_default()
        {
            qa_data::remove_qa_moderator(ctx, user.id, guild_id).await?;
            crate::success!(ctx, msg, "Removed {} from Q&A moderators", user.name);
        } else {
            qa_data::add_qa_moderator(ctx, user.id, guild_id).await?;
            crate::success!(ctx, msg, "Added {} to Q&A moderators", user.name);
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin, is_lotr_discord)]
#[aliases("answers")]
pub async fn qa_answer_channel(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.expect("Should be only used in guilds");

    if let Ok(channel_id) = args.parse::<ChannelId>() {
        qa_data::set_answer_channel(ctx, guild_id, channel_id).await?;
        crate::success!(
            ctx,
            msg,
            "Successfully set answer channel to <#{}>",
            channel_id
        );
    } else {
        crate::failure!(ctx, msg, "The first argument must be a channel mention!");
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin, is_lotr_discord)]
#[aliases("questions")]
pub async fn qa_question_channel(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.expect("Should be only used in guilds");

    if let Ok(channel_id) = args.parse::<ChannelId>() {
        qa_data::set_question_channel(ctx, guild_id, channel_id).await?;
        crate::success!(
            ctx,
            msg,
            "Successfully set question channel to <#{}>",
            channel_id
        );
    } else {
        crate::failure!(ctx, msg, "The first argument must be a channel mention!");
    }
    Ok(())
}

#[command]
#[checks(is_admin, is_lotr_discord)]
#[only_in(guilds)]
#[aliases("disable")]
pub async fn qa_disable(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.expect("Should be only used in guilds");

    qa_data::disable_qa(ctx, guild_id).await?;

    crate::success!(ctx, msg, "Successfully disabled Q&A on this server.");

    Ok(())
}

#[command("info")]
#[checks(is_admin, is_lotr_discord)]
#[only_in(guilds)]
pub async fn qa_summary(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.expect("Should be only used in guilds");

    let (answer_channel, question_channel, moderators) = join3(
        qa_data::get_answer_channel(ctx, guild_id),
        qa_data::get_question_channel(ctx, guild_id),
        qa_data::get_qa_moderator_list(ctx, guild_id),
    )
    .await;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Current Q&A Setup");
                e.colour(0x38ffd4);
                e.author(|a| a.name("LOTR Mod Q&A").icon_url(crate::constants::BOT_ICON));
                e.fields([
                    (
                        "Questions channel",
                        if let Some(channel) = question_channel {
                            channel.mention().to_string()
                        } else {
                            "None, set it up with `!q&a questions <channel mention>`.".to_string()
                        },
                        false,
                    ),
                    (
                        "Answer archive channel",
                        if let Some(channel) = answer_channel {
                            channel.mention().to_string()
                        } else {
                            "None, set it up with `!q&a answers <channel mention>`.".to_string()
                        },
                        false,
                    ),
                    (
                        "Q&A Moderators",
                        match moderators {
                            Some(mods) if !mods.is_empty() => mods
                                .iter()
                                .map(|user_id| user_id.mention().to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            _ => "None, add moderators with `!q&a moderator <user mention>`."
                                .to_string(),
                        },
                        false,
                    ),
                ]);
                e
            });
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_parse());
            m
        })
        .await?;

    Ok(())
}

#[command]
#[owners_only]
#[checks(is_admin)]
async fn qa_cache(ctx: &Context) -> CommandResult {
    let channels_cache = {
        let data_read = ctx.data.read().await;
        data_read.get::<qa_data::QaChannelsCache>().unwrap().clone()
    };
    println!(
        "=== Q&A CHANNELS CACHE ===\n{:?}\n=== END ===",
        channels_cache
    );
    Ok(())
}
