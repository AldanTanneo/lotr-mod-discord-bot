use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::{channel::Message, prelude::ReactionType, Permissions};
use serenity::utils::Colour;

use crate::check::{has_permission, IS_ADMIN_CHECK};
use crate::database::{
    admin_data::is_admin,
    config::{get_minecraft_ip, get_prefix},
    custom_commands::get_custom_commands_list,
};
use crate::OWNER_ID;

#[command]
#[sub_commands(json)]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let is_admin = msg.author.id == OWNER_ID
        || is_admin(ctx, msg.guild_id, msg.author.id).await.is_some()
        || has_permission(
            ctx,
            msg.guild_id,
            &msg.author,
            Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR,
        )
        .await;

    let prefix = get_prefix(ctx, msg.guild_id)
        .await
        .unwrap_or_else(|| "!".into());
    let is_minecraft_server = get_minecraft_ip(ctx, msg.guild_id).await.is_some();

    let cclist = get_custom_commands_list(ctx, msg.guild_id)
        .await
        .unwrap_or_default();
    let cctext = cclist
        .into_iter()
        .filter_map(|(name, desc)| {
            if !is_admin && desc.is_empty() {
                None
            } else {
                let desc = if desc.is_empty() {
                    "_No description_".into()
                } else {
                    desc
                };
                Some(format!("`{}{}`  {}\n", prefix, name, desc))
            }
        })
        .collect::<Vec<_>>()
        .join("");

    msg.author
        .direct_message(ctx, |m| {
            m.content(format!("My prefix here is \"{}\"", prefix));
            m.embed(|e| {
                e.colour(Colour::DARK_GREEN);
                e.title("Available commands");
                e.field(
                    "General commands",
                    format!(
"`{prefix}curseforge [legacy|renewed]`  Display the mod download link (default: `legacy`)
`{prefix}invite`  Send the bot invite link in DMs
`{prefix}help{json}`  Send this message in DMs
`{prefix}donate`  Display the mod donation links
`{prefix}facebook`  Display the mod Facebook page link
`{prefix}discord`  Display the invite link to the community discord

*Not available in DMs:*
`{prefix}renewed`  Technical support command
`{prefix}forge`  Technical support command
`{prefix}coremod`  Technical support command

",
                        prefix=prefix,
                        json=if is_admin {" [json]"} else {""}
                    ),
                    false,
                );
                if is_minecraft_server || is_admin {
                    e.field(
                        "Minecraft server commands",
                        format!(
"`{prefix}ip{}`  Display the server ip{}
`{prefix}online [ip]`  Display the server status and a list of online players (default: the server's set ip)
",
                            if is_admin {
                                " [set <server ip>]"
                            } else {
                                ""
                            },
                            if is_admin {
                                ", if it exists; use `set` to add one."
                            } else {
                                ""
                            },
                            prefix=prefix
                        ),
                        false
                    );
                }
                e.field(
                    "Wiki commands",
                    format!(
"`{prefix}wiki [language] <query>`  Display search result from the [LOTR Mod Wiki](https://lotrminecraftmod.fandom.com/)
(default language: `en`)
Available languages: `en`, `de`, `fr`, `es`, `nl`, `ja`, `zh`, `ru`

*Subcommands:*
`{prefix}wiki user [language] <user name>`
`{prefix}wiki category [language] <category name>`
`{prefix}wiki template [language] <template name>`
`{prefix}wiki file [language] <file name>`

`{prefix}wiki random`  Display a random wiki page (from the English wiki only)

`{prefix}wiki tolkien <query>`  Display search result from [TolkienGateway](http://www.tolkiengateway.net/)
`{prefix}wiki minecraft <query>`  Display search result from the [Official Minecraft Wiki](https://minecraft.gamepedia.com/)
",
                        prefix = prefix
                    ),
                    false
                );
                if !cctext.is_empty() {
                    e.field(
                        "Custom commands",
                        cctext,
                        false,
                    );
                }
                if is_admin {
                    e.field(
                        "Admin commands",
                        format!(
"`{prefix}prefix [new prefix]`  Display or change the bot prefix for your server
`{prefix}admin add <user mention>`  Give a user admin rights for the bot
`{prefix}admin remove <user mention>`  Removes admin rights for a user
`{prefix}admin list`  Display a list of bot admins
`{prefix}blacklist [user or channel mention]`  Prevent some commands to be used by the user or in the channel (except for bot admins). When used without arguments, displays the blacklist.
`{prefix}announce <channel mention> <json message contents>`  Make the bot send a message to the mentioned channel. For the JSON argument documentation, type `{prefix}help json`

`{prefix}define <command name> <json command content>`  Define or update a custom command.  For the JSON argument documentation, type `{prefix}help json`
`{prefix}command display [command name]`  Provide an argument to get info on a specific command, or leave empty to get a list of commands
`{prefix}command remove <command name>`  Remove a custom command

*Only bot admins can use these commands*
", prefix=prefix),
                        false,
                    );
                }
                e
            });
            m
        })
        .await?;

    msg.react(ctx, ReactionType::from('✅')).await?;

    Ok(())
}

#[command]
#[checks(is_admin)]
pub async fn json(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author
        .direct_message(ctx, |m| {
            m.content(
                r#"**JSON documentation for the announcement and define commands**
*Almost all fields are optional. Try it out!*
```json
{
    "content": "the message content",
    "image": "a valid image url",
    "reactions": [
        "🍎", // unicode emojis
        "<:name:0000000000000000>" // custom emojis
    ],
    "embed": {
        "colour": "RRGGBB", // hexadecimal color code
        "author": {
            "name": "the embed author name",
            "icon": "a valid author icon url",
            "url": "a valid url that will open when clicking on the author name"
        },
        "title": "the embed title",
        "url": "a valid url that will open when clicking on the title",
        "description": "the embed description",
        "image": "an embed image",
        "thumbnail": "a valid thumbnail image url",
        "fields": [ // a list of fields to display in the embed; an element looks like:
            [
                "a field title",
                "some field content",
                true // or false: wether the field is inlined or not 
                     // (if not, displays as a block)
            ]
        ],
        "footer" : {
            "icon": "a valid footer icon url",
            "text": "some footer text"
        },
        "timestamp": "a valid timestamp in the format [YYYY]-[MM]-[DD]T[HH]:[mm]:[ss]"
                     // example: "2020-12-02T13:07:00"

    }

    // Only for custom commands:
    "documentation": "A formatted string" 
        // if this field is not present, your custom command will not be
        // displayed in !help for regular users
    "type": "default" // can be "meme", "admin" or "default";
        // if the type is "meme", the command will be subject to the blacklist
        // if the type is "admin", only admins will be able to use it.
    "default_args": ["arg0", "arg1", ...]
        // if $0, $1 are left in the json because there are not enough arguments
        // to fill them, these values will be used.
}
```
"#,
            )
        })
        .await?;
    Ok(())
}
