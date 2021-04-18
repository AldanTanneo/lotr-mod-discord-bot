use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::{
    channel::Message,
    id::{ChannelId, MessageId},
};

use crate::announcement;
use crate::check::IS_ADMIN_CHECK;
use crate::utils::{get_json_from_message, JsonMessageError::*};
use crate::{failure, handle_json_error, success};

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[sub_commands("edit")]
pub async fn announce(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = serenity::utils::parse_channel(args.single::<String>()?.trim());
    if let Some(id) = channel {
        if msg.guild_id
            != ChannelId(id)
                .to_channel(ctx)
                .await?
                .guild()
                .map(|c| c.guild_id)
        {
            failure!(
                ctx,
                msg,
                "You can only announce in the same server as the one you are in!"
            );
            return Ok(());
        };
        let message = get_json_from_message(msg).await;
        match message {
            Ok(value) => {
                if announcement::announce(ctx, ChannelId(id), value)
                    .await
                    .is_ok()
                {
                    success!(ctx, msg);
                } else {
                    failure!(ctx, msg, "Error sending the message! Check your JSON content and/or the bot permissions.");
                }
            }
            Err(e) => handle_json_error!(ctx, msg, e),
        }
    } else {
        failure!(ctx, msg, "The first argument must be a channel mention!");
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[only_in(guilds)]
pub async fn edit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = serenity::utils::parse_channel(args.single::<String>()?.trim()).map(ChannelId);
    let msg_id = args.single::<u64>().ok();
    if let Some(channel_id) = channel {
        if msg.guild_id
            != channel_id
                .to_channel(ctx)
                .await?
                .guild()
                .map(|c| c.guild_id)
        {
            failure!(
                ctx,
                msg,
                "You can only announce in the same server as the one you are in!"
            );
            return Ok(());
        };
        if msg_id.is_some() && channel_id.message(ctx, msg_id.unwrap_or(0)).await.is_ok() {
            let message = get_json_from_message(msg).await;
            match message {
                Ok(value) => {
                    if announcement::edit_message(
                        ctx,
                        channel_id,
                        MessageId(msg_id.unwrap_or(0)),
                        value,
                    )
                    .await
                    .is_ok()
                    {
                        success!(ctx, msg);
                    }
                }
                Err(e) => handle_json_error!(ctx, msg, e),
            }
        } else {
            failure!(ctx, msg, "The second argument must be a message ID!");
        }
    } else {
        failure!(ctx, msg, "The first argument must be a channel mention!");
    }
    Ok(())
}
