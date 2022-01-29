pub mod curseforge;
pub mod minecraft;

use const_format::formatcp;
use poise::serenity::model::interactions::message_component::ButtonStyle;

use crate::constants;
use crate::serenity::CreateBotAuthParameters;
use crate::serenity::OAuth2Scope;
use crate::{Context, Result};

/// Display the invite link to the LOTR Mod Community discord
#[poise::command(slash_command)]
pub async fn discord(ctx: Context<'_>) -> Result {
    ctx.say(
        "The invite for the LOTR Mod Community Discord is available here:
https://discord.gg/QXkZzKU",
    )
    .await?;
    Ok(())
}

/// Display the link to the mod's Facebook page
#[poise::command(slash_command)]
pub async fn facebook(ctx: Context<'_>) -> Result {
    use constants::socials::facebook::*;

    ctx.send(|m| {
        m.embed(|e| {
            e.colour(COLOUR);
            e.description(formatcp!(
                "Check the mod's Facebook page for
updates and teasers [here]({URL})!"
            ));
            e.thumbnail(ICON);
            e.title("Link to the Facebook page");
            e.url(URL);
            e
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| b.style(ButtonStyle::Link).label("Facebook page").url(URL))
            })
        })
    })
    .await?;
    Ok(())
}

/// Display the link to the mod's Instagram page
#[poise::command(slash_command)]
pub async fn instagram(ctx: Context<'_>) -> Result {
    use constants::socials::instagram::*;
    ctx.send(|m| {
        m.embed(|e| {
            e.colour(COLOUR);
            e.description(formatcp!(
                "Check the mod's Instagram page for
updates and teasers [here]({URL})!"
            ));
            e.thumbnail(ICON);
            e.title("Link to the Instagram page");
            e.url(URL);
            e
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| b.style(ButtonStyle::Link).label("Instagram page").url(URL))
            })
        })
    })
    .await?;
    Ok(())
}

/// Display paypal links to make donations to the mod's creator
#[poise::command(slash_command)]
pub async fn donate(ctx: Context<'_>) -> Result {
    use constants::donate::*;

    ctx.send(|m| {
        m.embed(|e| {
            e.colour(COLOUR);
            e.description(
                "Donations of **£5 GBP** or over will be thanked with the Patron \
[Shield](https://lotrminecraftmod.fandom.com/wiki/Shield) & \
[Title](https://lotrminecraftmod.fandom.com/wiki/Title) in the next released update if you write \
your Minecraft username in the donation message.",
            );
            e.thumbnail(ICON);
            e.title("Donate to the mod!");
            e
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.style(ButtonStyle::Link)
                        .label("Donate in $")
                        .url(URL_DOLLARS)
                })
                .create_button(|b| {
                    b.style(ButtonStyle::Link)
                        .label("Donate in £")
                        .url(URL_POUNDS)
                })
                .create_button(|b| {
                    b.style(ButtonStyle::Link)
                        .label("Donate in €")
                        .url(URL_EUROS)
                })
            })
        })
    })
    .await?;
    Ok(())
}

/// Display the bot's invite link
#[poise::command(slash_command)]
pub async fn invite(ctx: Context<'_>) -> Result {
    let user_icon = ctx.discord().cache.current_user_field(|u| u.face());

    let invite_url = {
        let mut builder = CreateBotAuthParameters::default();
        builder
            .permissions(constants::INVITE_PERMISSIONS)
            .auto_client_id(ctx.discord())
            .await?
            .scopes(&[OAuth2Scope::Bot, OAuth2Scope::ApplicationsCommands]);
        builder.build()
    };

    ctx.send(|m| {
        m.embed(|e| {
            e.colour(crate::serenity::colours::branding::BLURPLE)
                .author(|a| {
                    a.name("LOTR Mod Bot");
                    a.icon_url(user_icon)
                })
                .field(
                    "Invite me to your server!",
                    format!("My invite link is available [here]({}).", invite_url),
                    false,
                )
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.style(ButtonStyle::Link)
                        .label("Invite me")
                        .url(invite_url)
                })
            })
        })
    })
    .await?;
    Ok(())
}

/// Get information on a user
#[poise::command(slash_command, context_menu_command = "User information", ephemeral)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "The user to get info on"] user: crate::serenity::User,
) -> Result {
    use crate::serenity::Mentionable;

    let member = if let Some(guild_id) = ctx.guild_id() {
        guild_id.member(ctx.discord(), user.id).await.ok()
    } else {
        None
    };

    let colour = member.as_ref().map(|m| m.colour(ctx.discord())).flatten();

    ctx.send(|m| {
        m.embed(|e| {
            if let Some(colour) = colour {
                e.colour(colour);
            }
            e.thumbnail(user.face());
            if let Some(nick) = member.as_ref().map(|mb| mb.nick.as_ref()).flatten() {
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
            if let Some(joined_at) = member.as_ref().map(|mb| mb.joined_at.as_ref()).flatten() {
                e.field(
                    "Account join date",
                    joined_at.format("%d %B %Y at %R"),
                    true,
                );
            }
            if let Some(roles) = member
                .as_ref()
                .map(|mb| (!mb.roles.is_empty()).then(|| &mb.roles))
                .flatten()
            {
                e.field(
                    "Roles",
                    roles
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
