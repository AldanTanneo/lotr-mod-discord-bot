use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use crate::check::IS_LOTR_DISCORD_CHECK;

#[command]
#[only_in(guilds)]
async fn renewed(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::GOLD);
                e.title("Use the 1.7.10 version");
                e.description(
                    "The 1.15.2 version of the mod is a work in progress, missing many features such as NPCs and structures.
You can find those in the full 1.7.10 Legacy edition [here](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-legacy)",
                )
            });

            m
        })
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_lotr_discord)]
async fn tos(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx,
            "This is the Discord server of the **Lord of the Rings Mod**, not the official Minecraft server of the mod.
Their Discord can be found here: https://discord.gg/gMNKaX6",
        )
        .await?;
    Ok(())
}

#[command]
async fn curseforge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let version = if args.single::<String>().unwrap_or_default() == "legacy" {
        "Legacy"
    } else {
        "Renewed"
    };
    msg.channel_id.send_message(ctx, |m| m.embed(|e|{
        e.colour(Colour::GOLD);
        e.title(format!("Link to the {} version", version));
        e.description(format!(
            "The {} edition of the mod can be found on [Curseforge](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-{})",
            version,
            version.to_lowercase()
        ))
    })).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn forge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (version, mc) = if args.single::<String>().unwrap_or_default() == "legacy" {
        ("1558", "1.7.10")
    } else {
        ("31.2.31", "1.15.2")
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
async fn coremod(ctx: &Context, msg: &Message) -> CommandResult {
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
