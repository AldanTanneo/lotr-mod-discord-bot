use bytesize::ByteSize;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::utils::Colour;

use crate::api::curseforge;
use crate::check::*;
use crate::constants::{CURSEFORGE_ID_LEGACY, CURSEFORGE_ID_RENEWED};

#[command]
#[only_in(guilds)]
#[aliases("legacy")]
pub async fn renewed(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_GOLD);
                e.title("Use the 1.7.10 version");
                e.description(
                    "The 1.16.5 version of the mod is a work in progress, missing many features such as NPCs and structures.
You can find those in the full 1.7.10 Legacy edition [here](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-legacy).

For a list of features present in the renewed version, check [this page](https://lotrminecraftmod.fandom.com/wiki/Updates/Renewed).",
                )
            });

            m
        })
        .await?;

    Ok(())
}

fn pretty_large_int<T: Into<u128>>(x: T) -> String {
    let mut num = x.into();
    let mut s = String::new();
    while num / 1000 != 0 {
        s = format!(",{:03}{}", num % 1000, s);
        num /= 1000;
    }
    format!("{}{}", num % 1000, s)
}

#[command]
#[aliases("download")]
pub async fn curseforge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let id = if args.single::<String>().unwrap_or_default().to_lowercase() == "renewed" {
        CURSEFORGE_ID_RENEWED
    } else {
        CURSEFORGE_ID_LEGACY
    };
    let project = curseforge::get_project_info(ctx, id).await;
    if let Some(project) = project {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.author(|a| {
                        a.name("Curseforge");
                        a.icon_url(
                            "https://pbs.twimg.com/profile_images/1334200314136817665/QOJeY7B0_400x400.png",
                        )
                    });
                    e.colour(Colour(0xf16436));
                    e.title(&project.title);
                    e.url(&project.urls.curseforge);
                    e.description(&project.summary);
                    e.thumbnail(&project.thumbnail);
                    e.field(
                        "Download link",
                        format!(
                            "[{}]({}) ({})",
                            &project.download.name,
                            &project.download.url,
                            ByteSize(project.download.filesize)
                        ),
                        false,
                    );
                    e.footer(|f| {
                        f.text(format!(
                            "Total download count: {}",
                            pretty_large_int(project.downloads.total)
                        ))
                    });
                    e.timestamp(project.download.uploaded_at);
                    e
                })
            })
            .await
            .unwrap();
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn forge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (version, mc) = if args.single::<String>().unwrap_or_default() == "legacy" {
        ("1558", "1.7.10")
    } else {
        ("36.1.0", "1.16.5")
    };
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.colour(Colour::DARK_BLUE);
            e.title("Have you checked your Forge version?");
            e.description(format!("To function properly, the mod needs to run with Forge {} or later for Minecraft {}", version, mc));
            e.author(|a| {
                a.name(format!("Minecraft Forge for {}", mc));
                a.icon_url("https://pbs.twimg.com/profile_images/778706890914095109/fhMDH9o6_400x400.jpg");
                a.url(format!("http://files.minecraftforge.net/maven/net/minecraftforge/forge/index_{}.html", mc))
            })
        })
    })
    .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn coremod(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| m.embed(|e| {
            e.colour(Colour::RED);
            e.title("Check your mod file extension!");
            e.description("Sometimes when downloading the mod with a browser like Firefox, the mod file is saved with a `.zip` extension instead of a `.jar`
When this happens, the mod will not function properly: among other things that will happen, mod fences and fence gates will not connect, and horses will go very slowly.

To fix this, go to your `/.minecraft/mods` folder and change the file extension!")
        }))
        .await?;
    Ok(())
}

#[command]
#[checks(allowed_blacklist)]
pub async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author.dm(ctx, |m| {
        m.embed(|e| {
            e.colour(Colour::BLURPLE);
            e.field("Invite me to your server!", "My invite link is available [here](https://github.com/AldanTanneo/lotr-mod-discord-bot/blob/main/README.md)", false)
        })
    }).await?;

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Sent invite link to DMs!").await?;
    }

    Ok(())
}

#[command]
pub async fn discord(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            "The invite for the **LOTR Mod Community Discord** is available here:
https://discord.gg/QXkZzKU",
        )
        .await?;
    Ok(())
}

#[command]
#[aliases("fb")]
pub async fn facebook(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.colour(Colour::new(0x3B5998));
        e.description("Check the mod’s Facebook page for\nupdates and teasers [here](https://www.facebook.com/LOTRMC)!");
        e.thumbnail("https://i.ibb.co/rdJtpWY/10610821-779526068752432-5484491658565693801-n.jpg");
        e.title("Link to the Facebook page");
        e.url("https://www.facebook.com/LOTRMC");
        e
    })).await?;
    Ok(())
}

#[command]
#[aliases("donation", "paypal")]
pub async fn donate(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.colour(Colour::new(0xCEBD9C));
        e.description("Donations of **£5 GBP** or over will be thanked with the Patron [Shield](https://lotrminecraftmod.fandom.com/wiki/Shield) & [Title](https://lotrminecraftmod.fandom.com/wiki/Title) in the next released update if you write your Minecraft username in the donation message.");
        e.field("Donate in $", "[Paypal](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=YZ97X6UBJJD7Y)", true);
        e.field("Donate in £", "[Paypal](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=8BXR2F4FYYEK2)", true);
        e.field("Donate in €", "[Paypal](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=5Q4NK7C5N2FB4)", true);
        e.thumbnail("https://media.discordapp.net/attachments/781837314975989772/809773869971013653/Donate.png");
        e.title("Donate to the mod!");
        e
    })).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[aliases(user)]
pub async fn user_info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_id = if let Some(user) = msg.mentions.first() {
        user.id
    } else if let Ok(user_id) = args.single::<UserId>() {
        user_id
    } else {
        msg.author.id
    };
    let member = msg
        .guild_id
        .unwrap_or_default()
        .member(ctx, user_id)
        .await?;
    let user = &member.user;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::PURPLE);
                e.thumbnail(user.face());
                if let Some(nick) = &member.nick {
                    e.title(nick);
                    e.description(format!(
                        "Username: **{}**{}",
                        &user.name,
                        if user.bot {
                            "\n_This user is a bot_"
                        } else {
                            ""
                        }
                    ));
                } else {
                    e.title(&user.name);
                    if user.bot {
                        e.description("_This user is a bot_");
                    }
                }
                e.field(
                    "Account creation date",
                    &user.id.created_at().format("%d %B %Y at %R"),
                    true,
                );
                if let Some(joined_at) = member.joined_at {
                    e.field(
                        "Account join date",
                        joined_at.format("%d %B %Y at %R"),
                        true,
                    );
                }
                if !member.roles.is_empty() {
                    e.field(
                        "Roles",
                        member
                            .roles
                            .iter()
                            .map(|r| r.mention().to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        false,
                    );
                }
                e
            })
        })
        .await?;
    Ok(())
}
