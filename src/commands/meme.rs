use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;

use crate::check::*;
use crate::constants::{BIT_FILTER_24BITS, OWNER_ID};
use crate::database::floppa::{add_floppa, get_floppa, is_floppadmin};
use crate::success;
use crate::utils::NotInGuild;

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[bucket = "basic"]
async fn floppa(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let n = args.single::<i64>().ok();
    let url = if let Some(url) = get_floppa(ctx, n).await {
        url
    } else {
        "https://i.kym-cdn.com/photos/images/original/001/878/839/c6f.jpeg".to_string()
    };
    msg.reply(ctx, url).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[bucket = "basic"]
async fn aeugh(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file("https://cdn.discordapp.com/attachments/405122337139064834/782087543046668319/aeugh.mp4");
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_parse())
        })
        .await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[bucket = "basic"]
async fn dagohon(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file("https://cdn.discordapp.com/attachments/405097997970702337/782656209987043358/dagohon.mp4");
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_parse())
        })
        .await?;
    Ok(())
}

#[command]
#[checks(allowed_blacklist)]
#[bucket = "basic"]
async fn colour(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let colour_value = args
        .current()
        .map(|s| u32::from_str_radix(s.trim_start_matches('#'), 16).ok())
        .flatten()
        .unwrap_or_else(|| (&args as *const Args) as u32 & BIT_FILTER_24BITS);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(format!("Random colour #{:06x}", colour_value));
                e.image(format!(
                    "https://singlecolorimage.com/get/{:06x}/400x300",
                    colour_value
                ));
                e.colour(colour_value);
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn floppadd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if msg.author.id == OWNER_ID
        || is_floppadmin(ctx, server_id, msg.author.id)
            .await
            .unwrap_or_default()
    {
        let url = args.single::<String>();
        if let Ok(floppa_url) = url {
            let owner = OWNER_ID.to_user(ctx).await?;
            let guild = server_id
                .to_partial_guild(ctx)
                .await
                .map(|g| g.name)
                .unwrap_or_else(|_| "DMs".to_string());

            let dm = owner
                .dm(ctx, |m| {
                    m.content(format!(
                        "Floppa added by {} in {}\n{}",
                        msg.author.name, guild, &floppa_url
                    ))
                })
                .await?;

            add_floppa(ctx, floppa_url.clone()).await?;

            success!(ctx, dm);
            success!(ctx, msg);
        }
    }
    Ok(())
}
