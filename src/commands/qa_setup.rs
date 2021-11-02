use serenity::framework::standard::{macros::command, Args, CommandResult};
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
