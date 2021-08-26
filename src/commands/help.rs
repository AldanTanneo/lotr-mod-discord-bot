use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::Colour;

use crate::check::*;
use crate::constants::{MANAGE_BOT_PERMS, OWNER_ID};
use crate::database::{
    config::{get_minecraft_ip, get_prefix},
    custom_commands::get_custom_commands_list,
};
use crate::is_admin;
use crate::utils::has_permission;

#[command]
#[aliases("commands")]
#[sub_commands(json, custom_commands, bugtracker, admin_help)]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let server_id = msg.guild_id.unwrap_or_default();
    let is_admin = msg.author.id == OWNER_ID
        || is_admin!(ctx, msg)
        || has_permission(ctx, server_id, msg.author.id, MANAGE_BOT_PERMS).await;

    let prefix = get_prefix(ctx, server_id)
        .await
        .unwrap_or_else(|| "!".into());
    let is_minecraft_server = get_minecraft_ip(ctx, server_id).await.is_some();

    let cclist = get_custom_commands_list(ctx, server_id)
        .await
        .unwrap_or_default();
    let mut newline: u32 = 0;
    let cctext = cclist
        .into_iter()
        .filter_map(|(name, desc)| {
            if !is_admin && desc.is_empty() {
                None
            } else {
                if desc.is_empty() {
                    newline += 1;
                }
                Some(format!(
                    "{newline}`{}{}`{}",
                    prefix,
                    name,
                    match newline {
                        0 => format!("  {}\n", desc),
                        _ => String::new(),
                    },
                    newline = match newline {
                        0 => "",
                        1 => {
                            newline += 1;
                            "\n"
                        }
                        _ => ", ",
                    }
                ))
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
                    "**General commands**",
                    format!(
"`{prefix}curseforge [legacy|renewed]`  Display the mod download link (default: `legacy`)
`{prefix}invite`  Send the bot invite link
`{prefix}help{json}`  Send this message in DMs
`{prefix}donate`  Display the mod donation links
`{prefix}facebook`  Display the mod Facebook page link
`{prefix}instagram`  Display the mod Instagram page link
`{prefix}discord`  Display the invite link to the community discord

*Not available in DMs:*
`{prefix}renewed`  Technical support command
`{prefix}forge`  Technical support command
`{prefix}coremod`  Technical support command
`{prefix}user`  Display information about a user
`{prefix}role <role name>`  Claim the given role. The role has to be explicitly \
defined by admins of the server. Use  `{prefix}roles`  to see a list of available roles.",
                        prefix=prefix,
                        json=if is_admin {" [json]"} else {""}
                    ),
                    false,
                );
                e.field(
                    "**Wiki commands**",
                    format!(
                        "`{prefix}wiki [language] <query>`  Display search result from the \
[LOTR Mod Wiki](https://lotrminecraftmod.fandom.com/)
(default language: `en`)
Available languages: `en`, `de`, `fr`, `es`, `nl`, `ja`, `zh`, `ru`

*Subcommands:*
`{prefix}wiki user [language] <user name>`
`{prefix}wiki category [language] <category name>`
`{prefix}wiki template [language] <template name>`
`{prefix}wiki file [language] <file name>`

`{prefix}wiki random`  Display a random wiki page (from the English wiki only)

`{prefix}wiki tolkien <query>`  Display search result from \
[TolkienGateway](http://www.tolkiengateway.net/)
`{prefix}wiki minecraft <query>`  Display search result from the \
[Official Minecraft Wiki](https://minecraft.gamepedia.com/)
",
                        prefix = prefix
                    ),
                    false,
                );
                if !cctext.is_empty() || is_admin {
                    e.footer(|f| f.text("1/2"));
                }
                e
            });
            m
        })
        .await?;

    if !cctext.is_empty() || is_admin {
        msg.author
            .direct_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(Colour::DARK_GREEN);
                    if is_minecraft_server || is_admin {
                        e.field(
                            "**Minecraft server commands**",
                            format!(
                                "`{prefix}ip{}`  Display the server ip{}
`{prefix}online [ip]`  Display the server status and a list of online players \
(default: the server's set ip)
",
                                if is_admin { " [set <server ip>]" } else { "" },
                                if is_admin {
                                    ", if it exists; use `set` to add one."
                                } else {
                                    ""
                                },
                                prefix = prefix
                            ),
                            false,
                        );
                    }
                    if !cctext.is_empty() {
                        e.field("**Custom commands**", cctext, false);
                    }
                    if is_admin {
                        e.field(
                            "Admin commands",
                            format!("Do  `{}help admin`  to see admin commands", prefix),
                            false,
                        );
                    }
                    e.footer(|f| f.text("2/2"));
                    e
                });
                m
            })
            .await?;
    }

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Help message sent to DMs!").await?;
    }

    Ok(())
}

#[command]
#[checks(is_admin)]
async fn json(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author
        .direct_message(ctx, |m| {
            m.content(
                r#"**JSON documentation for the announcement command**
*Almost all fields are optional. Try it out!*
*For custom commands documentation, use the command `help custom`.*
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
}
```
"#,
            )
        })
        .await?;

    if msg.guild_id.is_some() {
        msg.reply(ctx, "JSON help message sent to DMs!").await?;
    }

    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("custom")]
async fn custom_commands(ctx: &Context, msg: &Message) -> CommandResult {
    msg.author
        .direct_message(ctx, |m| {
            m.content(
                r#"**JSON documentation for custom commands**
*These fields are exclusive to custom commands. To add content to your custom command, see `help json`.*
```json
{
	"documentation": "A formatted string"
		// if this field is not present, your custom command will not be
		// displayed in !help for regular users
	"type": "default" // can be "meme", "admin" or "default";
		// if the type is "meme", the command will be subject to the blacklist
		// if the type is "admin", only admins will be able to use it.
	"default_args": ["arg0", "arg1", ...]
		// if $0, $1 are left in the json because there are not enough arguments
		// to fill them, these values will be used.
	"self_delete": true // or false: wether the command message is deleted after execution.
	"subcommands" : {
		"subcommand_name": {"content": "some content", ...},
		"other_subcommand_name": {...}, // define subcommands. 
			// They can override the 'type' and 'self_delete' tags,
			// but all the other tags must be redefined.
			// They do not show up in  `!help`, so, you need to mention
			// them in the main "documentation" tag.
		"alias_name": "subcommand_name" // calling this subcommand will call
			// the given existing subcommand.
	}
}
```
"#,
            )
        })
        .await?;

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Custom commands help message sent to DMs!")
            .await?;
    }

    Ok(())
}

#[command]
#[checks(is_admin)]
pub async fn bugtracker(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_prefix(ctx, msg.guild_id.unwrap_or_default())
        .await
        .unwrap_or_else(|| "!".into());
    msg.author
        .dm(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_GREEN);
                e.author(|a| {
                    a.icon_url(crate::constants::TERMITE_IMAGE);
                    a.name("The bugtracker is only available in the LOTR Mod Community Discord");
                    a
                });
                e.title("Available bugtracker commands");
                e.field(
                    "**Creating a bug report**",
                    format!(
"`{prefix}track [status] <bug title>`  Creates a new bug report with the optional specified \
`status`: one of `low`, `medium`, `high`, `critical`, and `forge` or `vanilla`. \
The command returns a unique bug id.
 \t**Must be used with an inline reply to a message that will constitute the \
 initial bug report content.**
\tYou can optionnally use  `{prefix}track legacy [status] <bug title>`  \
to create a legacy bug report.
`{prefix}bug link <bug id> [link url] [link title]`  Adds additional information to the bug \
report referenced by its `bug id`. Can also be used with an inline reply to a message, \
in which case you don't need to specify a url.
 \tThe command returns a unique link id which you can remove with the command  \
 `{prefix}bug link remove <bug id> <link id>`.
",
                        prefix = prefix,
                    ),
                    false,
                );
                e.field(
                    "**Displaying and editing bug reports**",
                    format!(
"`{prefix}bugs [latest|oldest|highest|lowest] [status] [page] [limit n]`  Displays a list of \
bugs. By default, it will display all bugs that are not `resolved`, `forge` or `closed`, in \
chronological order starting from the latest one, and with a default limit of 10 bugs.
 \tThe `limit` keyword is necessary to specify a custom limit. `highest` and `lowest` will \
 sort the bugs by priority.
 \tAvailable statuses are `low`, `medium`, `high`, `critical`, `resolved`, `forge` \
 (or `vanilla`) and `closed`.
 \tYou can optionnally use  `{prefix}bugs [legacy|renewed] [latest|oldest] [status] [limit]`  \
 to display legacy only or renewed only bugs.
`{prefix}bug <bug id>`  Displays a single bug.
`{prefix}bug rename <bug id> <new title>`  Change a bug's title.
`{prefix}bug status <bug id> <new status>`  Change a bug's status.
`{prefix}bug toggle <bug id>`  Switch a bug's edition between renewed and legacy.

`{prefix}bug statistics` Show bugtracker statistics.
",
                        prefix = prefix
                    ),
                    false,
                );
                e.field(
                    "**Closing a bug report**",
                    format!(
                        "`{prefix}resolve <bug id>`  Marks a bug as resolved. \
Equivalent to  `{prefix}bug status <bug id> resolved`.
`{prefix}bug close <bug id>`  Marks a bug as closed. \
Equivalent to  `{prefix}bug status <bug id> closed`.
",
                        prefix = prefix,
                    ),
                    false,
                );
                e
            })
        })
        .await?;

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Bugtracker help message sent to DMs!")
            .await?;
    }
    Ok(())
}

#[command]
#[checks(is_admin)]
#[aliases("admin")]
async fn admin_help(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_prefix(ctx, msg.guild_id.unwrap_or_default())
        .await
        .unwrap_or_else(|| "!".into());

    msg.author
        .dm(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::DARK_GREEN);
                e.title("Available commands");
                e.field(
                    "**General purpose commands**",
                    format!(
"`{prefix}prefix [new prefix]`  Display or change the bot prefix for your server
`{prefix}admin add <user mention>`  Give a user admin rights for the bot
`{prefix}admin remove <user mention>`  Removes admin rights for a user
`{prefix}admin list`  Display a list of bot admins
`{prefix}blacklist [user or channel mention]`  Prevent some commands to be used by the user or \
in the channel (except for bot admins). When used without arguments, displays the blacklist.", 
                        prefix=prefix
                    ),
                    false,
                );

                e.field(
                    "**Role commands**",
                    format!(
"`{prefix}role add <role mention> [role json properties]`  Define a new role for the  \
`{prefix}role`  command. All fields in the role JSON are optional.
```json
{{
    \"aliases\": [\"a list\", \"of aliases\"],
    \"time_requirement\": \"7days\", // a duration, written in a human readable format
    \"required_roles\": [\"a list\", \"of role names\"],
    \"incompatible_roles\": [\"a list\", \"of role names\"]
}}
```
`{prefix}role remove <role mention>`  Delete a role from the bot. This will not delete the role \
itself.
`{prefix}role show <role name>`  Display a role and its properties.",
                        prefix=prefix
                    ),
                    false,
                );

                e.field(
                    "**Annoucements & Custom commands**",
                    format!(
"`{prefix}announce <channel mention> <json message content>`  Make the bot send a \
message to the mentioned channel.  For the JSON argument documentation, type `{prefix}help json`

`{prefix}define <command name> <json command content>`  Define or update a custom command. \
For the JSON argument documentation, type  `{prefix}help custom`
`{prefix}command display [command name]`  Provide an argument to get info on a specific command, \
or leave empty to get a list of commands
`{prefix}command remove <command name>`  Remove a custom command

*Only bot admins can use these commands*
*For bugtracker help, use  `{prefix}help bugtracker`*",
                        prefix=prefix
                    ),
                    false,
                )
            })
        })
        .await?;

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Admin help message sent to DMs!").await?;
    }

    Ok(())
}
