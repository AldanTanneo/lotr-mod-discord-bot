use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::http::error::{DiscordJsonError, DiscordJsonSingleError, ErrorResponse};
use serenity::model::prelude::*;
use serenity::prelude::{HttpError, SerenityError};

use crate::announcement;
use crate::check::*;
use crate::constants::OWNER_ID;
use crate::utils::get_json_from_message;
use crate::{failure, handle_json_error, success};

async fn announcement_error_handler(
    ctx: &Context,
    msg: &Message,
    error: &CommandError,
) -> CommandResult {
    if let Some(SerenityError::Http(http_error)) = error.downcast_ref::<SerenityError>() {
        match http_error.as_ref() {
            HttpError::UnsuccessfulRequest(ErrorResponse {
                error:
                    DiscordJsonError {
                        code,
                        message,
                        errors,
                        ..
                    },
                ..
            }) => {
                msg.channel_id
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.author(|a| a.name("Error sending announcement"));
                            e.colour(serenity::utils::Colour::RED);
                            e.title(message);
                            e.description(format!("Error code: `{}`", code));
                            for DiscordJsonSingleError {
                                code,
                                message,
                                path,
                            } in errors
                            {
                                e.field(
                                    format!("`{}`", code),
                                    format!("{}\nPath: `{}`", message, path),
                                    false,
                                );
                            }
                            e
                        })
                    })
                    .await?;
            }
            _ => {
                failure!(
                ctx,
                msg,
                "Error sending/editing the message! Check your JSON content and/or the bot permissions."
            );
            }
        }
    } else {
        failure!(
                ctx,
                msg,
                "Error sending/editing the message! Check your JSON content and/or the bot permissions."
            );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[sub_commands("edit")]
pub async fn announce(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let channel = args.parse::<ChannelId>();
    if let Ok(channel_id) = channel {
        if msg.author.id != OWNER_ID
            && msg.guild_id
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
        let message = get_json_from_message(msg).await;
        match message {
            Ok(value) => {
                if let Err(error) = announcement::announce(ctx, channel_id, &value).await {
                    announcement_error_handler(ctx, msg, &error).await?;
                    return Err(error);
                } else {
                    println!(
                        "Annoucement by {} {:?} in {:?}, {}: {}",
                        msg.author.tag(),
                        msg.author.id,
                        channel_id,
                        msg.guild_id
                            .map(|id| format!("{:?}", id))
                            .unwrap_or_else(|| "None".into()),
                        value.to_string()
                    );
                    success!(ctx, msg);
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
    let channel = args.single::<ChannelId>();
    let msg_id = args.single::<u64>().ok();
    if let Ok(channel_id) = channel {
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
                "You can only edit announcements in the same server as the one you are in!"
            );
            return Ok(());
        };
        if msg_id.is_some() && channel_id.message(ctx, msg_id.unwrap_or(0)).await.is_ok() {
            let message = get_json_from_message(msg).await;
            match message {
                Ok(value) => {
                    if let Err(error) = announcement::edit_message(
                        ctx,
                        channel_id,
                        MessageId(msg_id.unwrap_or(0)),
                        &value,
                    )
                    .await
                    {
                        announcement_error_handler(ctx, msg, &error).await?;
                        return Err(error);
                    } else {
                        println!(
                            "Announcement edited by {} {} in {}, {}: {}",
                            msg.author.tag(),
                            msg.author.id,
                            channel_id,
                            msg.guild_id
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| "None".into()),
                            value.to_string()
                        );
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
