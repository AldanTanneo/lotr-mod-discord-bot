use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::{channel::Message, error::Error::WrongGuild, prelude::ReactionType};

use crate::check::ALLOWED_BLACKLIST_CHECK;
use crate::constants::OWNER_ID;
use crate::database::floppa::{add_floppa, get_floppa, is_floppadmin};
use crate::success;

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
async fn floppadd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.author.id == OWNER_ID
        || is_floppadmin(ctx, msg.guild_id, msg.author.id)
            .await
            .unwrap_or(false)
    {
        let owner = OWNER_ID.to_user(ctx);
        let guild = msg.guild_id.ok_or(WrongGuild)?.to_partial_guild(ctx);
        let url = args.single::<String>();
        if let Ok(floppa_url) = url {
            let owner = owner.await?;
            let guild = guild
                .await
                .map(|g| g.name)
                .unwrap_or_else(|_| "DMs".to_string());
            let dm = owner.dm(ctx, |m| {
                m.content(format!(
                    "Floppa added by {} in {}\n{}",
                    msg.author.name, guild, &floppa_url
                ))
            });
            add_floppa(ctx, floppa_url.clone()).await?;
            dm.await?.react(ctx, ReactionType::from('✅')).await?;
            success!(ctx, msg);
        }
    }
    Ok(())
}
