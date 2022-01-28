pub mod curseforge;
pub mod minecraft;

use crate::constants;
use crate::serenity::model::interactions::message_component::ButtonStyle;
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
