use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;

use crate::check::{IS_ADMIN_CHECK, IS_LOTR_DISCORD_CHECK};
use crate::database::bug_reports::{
    add_bug_report, add_link, change_bug_status, change_title, get_bug_from_id, get_bug_list,
    remove_link, BugStatus,
};
use crate::{failure, success};

#[command]
#[checks(is_admin, is_lotr_discord)]
#[aliases(report)]
pub async fn track(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let legacy = args.current().map(|s| s == "legacy").unwrap_or_default();
    if legacy {
        args.advance();
    }
    let tmp = args.single::<String>().unwrap_or_default();
    let status = tmp.as_str().into();
    if status == BugStatus::Medium && tmp.to_lowercase() != "medium" {
        args.rewind();
    }

    let title = args.rest();
    if title.is_empty() {
        failure!(ctx, msg, "You must provide a title for the bug report!");
        return Ok(());
    }

    let referenced_message = if let Some(message) = &msg.referenced_message {
        message
    } else {
        failure!(ctx, msg, "You must reference a message in your bug report!");
        return Ok(());
    };

    if let Some(bug_id) =
        add_bug_report(ctx, referenced_message, title.to_string(), status, legacy).await
    {
        success!(
            ctx,
            msg,
            "Tracking bug LOTR-{} (priority: `{:?}`)",
            bug_id,
            status
        );
        referenced_message
            .react(
                ctx,
                ReactionType::from(EmojiIdentifier {
                    animated: false,
                    id: EmojiId(839479605467152384),
                    name: "bug".into(),
                }),
            )
            .await?;
    } else {
        failure!(ctx, msg, "Could not submit the bug report!");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord)]
#[aliases(bugs)]
pub async fn buglist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let legacy = args
        .current()
        .map(|s| match s {
            "legacy" => Some(true),
            "renewed" => Some(false),
            _ => None,
        })
        .unwrap_or_default();
    if legacy.is_some() {
        args.advance();
    }
    let tmp = args.single::<String>().unwrap_or_default();
    let mut status: Option<BugStatus> = Some(tmp.as_str().into());
    if status == Some(BugStatus::Medium) && tmp.to_lowercase().as_str() != "medium" {
        status = None;
        args.rewind();
    }
    let ascending = args
        .single::<String>()
        .map(|s| s == "latest")
        .unwrap_or_default();
    let limit = args.single::<u32>().unwrap_or(10);
    if let Some(bugs) = get_bug_list(ctx, status, limit, ascending, legacy).await {
        if let Some(status) = status {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.author(|a| {
                                a.name("LOTR Mod Bugtracker");
                                a.icon_url("https://media.discordapp.net/attachments/781837314975989772/839479742457839646/termite.png");
                                a
                        });
                        e.colour(status.colour());
                        e.field(
                            format!(
                                "Bug reports (Status: {:?}){}", 
                                status, 
                                if let Some(b) = legacy {
                                    if b {
                                        " [legacy]"
                                    } else {
                                        " [renewed]"
                                    }
                                } else {
                                    ""
                                }
                            ),
                            if bugs.is_empty() {
                                "_No bugs with this status!_".to_string()
                            } else {
                                bugs.iter()
                                    .map(|b|
                                        format!(
                                            "{}{}",
                                            b,
                                            if legacy.is_none() && b.legacy {
                                                " [legacy]"
                                            } else {
                                                ""
                                            }
                                        )
                                    )
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            },
                            false,
                        );
                        e
                    })
                })
                .await?;
        } else {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.author(|a| {
                                a.name("LOTR Mod Bugtracker");
                                a.icon_url("https://media.discordapp.net/attachments/781837314975989772/839479742457839646/termite.png");
                                a
                        });
                        e.colour(serenity::utils::Colour::LIGHT_GREY);
                        e.field(
                            format!(
                                "Bug reports{}",
                                if let Some(b) = legacy {
                                    if b {
                                        " [legacy]"
                                    } else {
                                        " [renewed]"
                                    }
                                } else {
                                    ""
                                }
                            ),
                            if bugs.is_empty() {
                                "_No bugs registered!_".to_string()
                            } else {
                                bugs.iter()
                                    .map(|b| format!(
                                        "{}  `{:?}`{}",
                                        b, 
                                        b.status,
                                        if legacy.is_none() && b.legacy {
                                            " [legacy]"
                                        } else {
                                            ""
                                        }
                                    ))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            },
                            false,
                        );
                        e
                    })
                })
                .await?;
        }
    } else {
        failure!(ctx, msg, "Could not get bug list!");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord)]
#[sub_commands(track, bug_status, resolve, bug_close, bug_link, bug_rename)]
pub async fn bug(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            if let Some(bug) = get_bug_from_id(ctx, bug_id).await {
                let linked_message = bug.channel_id.message(ctx, bug.message_id).await;
                msg.channel_id
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.author(|a| {
                                a.name("LOTR Mod Bugtracker");
                                a.icon_url("https://media.discordapp.net/attachments/781837314975989772/839479742457839646/termite.png");
                                a
                            });
                            e.colour(bug.status.colour());
                            e.title(format!(
                                "LOTR-{}: {}{}",
                                bug_id,
                                bug.title,
                                if bug.legacy {
                                    " [legacy]"
                                } else {
                                    ""
                                }
                            ));
                            if let Ok(message) = linked_message {
                                e.description(message.content);
                                if let Some(image) = message.attachments.get(0) {
                                    e.image(&image.url);
                                }
                            }
                            if !bug.links.is_empty() {
                                e.field(
                                    "Additional information",
                                    &bug
                                        .links
                                        .iter()
                                        .fold(
                                            String::new(),
                                            |acc, link| format!("[{}]({}) (#{})\n{}", link.2, link.1, link.0, acc)
                                        ),
                                    false
                                );
                            }
                            e.footer(|f| {
                                f.text(format!("{} Status: {:?}", bug.status.icon(), bug.status))
                            });
                            e.timestamp(&bug.timestamp);
                            e
                        })
                    })
                    .await?;
            } else {
                failure!(ctx, msg, "Bug LOTR-{} does not exist!", bug_id);
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord, is_admin)]
pub async fn bug_status(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            if let Ok(status) = args.single::<String>().map(|s| s.to_lowercase()) {
                let new_status: BugStatus = status.as_str().into();
                if new_status == BugStatus::Medium && status != "medium" {
                    failure!(ctx, msg, "`{}` is not a valid bug status!", status);
                } else {
                    if let Some(old_status) = change_bug_status(ctx, bug_id, new_status).await {
                        success!(
                            ctx,
                            msg,
                            "Status changed for LOTR-{} from {:?} to {:?}!",
                            bug_id,
                            old_status,
                            new_status
                        );
                    } else {
                        failure!(ctx, msg, "The bug LOTR-{} does not exist!", bug_id);
                    }
                }
            } else {
                failure!(ctx, msg, "The second argument must be a bug status.");
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord, is_admin)]
pub async fn resolve(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            if change_bug_status(ctx, bug_id, BugStatus::Resolved)
                .await
                .is_some()
            {
                success!(ctx, msg, "LOTR-{} has been marked as resolved.", bug_id);
            } else {
                failure!(ctx, msg, "The bug LOTR-{} does not exist!", bug_id);
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord, is_admin)]
pub async fn bug_close(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            if change_bug_status(ctx, bug_id, BugStatus::Closed)
                .await
                .is_some()
            {
                success!(ctx, msg, "LOTR-{} has been marked as closed.", bug_id);
            } else {
                failure!(ctx, msg, "The bug LOTR-{} does not exist!", bug_id);
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord, is_admin)]
#[sub_commands(bug_link_remove)]
pub async fn bug_link(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            if let Some(message) = &msg.referenced_message {
                let title = args.rest();
                if title.is_empty() {
                    failure!(ctx, msg, "Specify a title for your message link!");
                    return Ok(());
                }
                if let Some(link_id) = add_link(ctx, bug_id, &message.link(), title).await {
                    success!(ctx, msg, "Added link #{} to LOTR-{}", link_id, bug_id);
                } else {
                    failure!(ctx, msg, "LOTR-{} does not exist!", bug_id);
                }
            } else {
                if let Ok(link) = args.single::<String>() {
                    let title = args.rest();
                    if title.is_empty() {
                        failure!(ctx, msg, "Specify a title for your message link!");
                        return Ok(());
                    }
                    if let Some(link_id) = add_link(ctx, bug_id, &link, title).await {
                        success!(ctx, msg, "Added link #{} to LOTR-{}", link_id, bug_id);
                    } else {
                        failure!(ctx, msg, "LOTR-{} does not exist!", bug_id);
                    }
                } else {
                    failure!(ctx, msg, "You need to either reference a message or specify a link to add to the bug report.");
                }
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_admin, is_lotr_discord)]
#[aliases(remove)]
pub async fn bug_link_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            let link_id = args.single::<String>();
            if let Ok(link_id) = link_id {
                if let Ok(link_id) = link_id.trim_start_matches("#").parse::<u64>() {
                    if remove_link(ctx, bug_id, link_id).await.is_ok() {
                        success!(
                            ctx,
                            msg,
                            "Successfully removed link #{} from LOTR-{}",
                            link_id,
                            bug_id
                        );
                    } else {
                        failure!(
                            ctx,
                            msg,
                            "Link #{} does not exist in LOTR-{}",
                            link_id,
                            bug_id
                        );
                    }
                } else {
                    failure!(ctx, msg, "`{}` is not a valid link id!", link_id);
                }
            } else {
                failure!(ctx, msg, "The second argument must be a valid link id.");
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}

#[command]
#[checks(is_lotr_discord, is_admin)]
pub async fn bug_rename(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(bug_id) = args.single::<String>() {
        if let Ok(bug_id) = bug_id
            .to_uppercase()
            .trim_start_matches("LOTR-")
            .parse::<u64>()
        {
            let new_title = args.rest();
            if new_title.is_empty() {
                failure!(ctx, msg, "You must specify a new title for LOTR-{}", bug_id);
            } else {
                if change_title(ctx, bug_id, new_title).await.is_ok() {
                    success!(
                        ctx,
                        msg,
                        "Successfully changed the title of LOTR-{}",
                        bug_id
                    );
                } else {
                    failure!(ctx, msg, "LOTR-{} does not exist!", bug_id);
                }
            }
        } else {
            failure!(ctx, msg, "`{}` is not a valid bug id!", bug_id);
        }
    } else {
        failure!(ctx, msg, "The first argument must be a bug id.");
    }
    Ok(())
}