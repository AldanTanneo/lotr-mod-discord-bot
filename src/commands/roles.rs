use chrono::{DateTime, Duration, Utc};
use humantime_serde::re::humantime::format_duration;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::prelude::*;

use crate::check::*;
use crate::constants::OWNER_ID;
use crate::database::roles;
use crate::utils::{get_json_from_message, has_permission, NotInGuild};
use crate::{failure, handle_json_error, is_admin, role_cache, success, warn};

use Reason::*;

macro_rules! role_message {
    ($ctx:ident, $msg:ident, $role:ident, $single_message:expr) => {
        $msg.author
            .direct_message($ctx, |m| {
                m.embed(|e| {
                    e.description($single_message)
                    .colour($role.colour)
                })
            })
            .await?;
    };
    ($ctx:ident, $msg:ident, $role:ident, $($message:tt)*) => {
        role_message!($ctx, $msg, $role, format!($($message)*))
    };
}

macro_rules! role_log {
    ($msg:ident, $role:ident, $log:literal) => {
        println!(
            $log,
            role_name = $role.name,
            role_id = $role.id,
            user_name = $msg.author.name,
            user_id = $msg.author.id
        );
    };
    ($msg:ident, $role:ident, $log:literal, $($extra:tt)*) => {
        println!(
            $log,
            $($extra)*,
            role_name = $role.name,
            role_id = $role.id,
            user_name = $msg.author.name,
            user_id = $msg.author.id,
        );
    };
}

#[inline]
pub fn format_role_name(name: &str) -> String {
    name.to_lowercase().replace(&['-', '_'][..], " ")
}

#[derive(Debug, Clone)]
enum Reason<'a> {
    NotEnoughTime(DateTime<Utc>),
    IncompatibleRole(&'a str),
    MissingRequiredRole(&'a str),
    TimeConversionError,
    RoleRetrievalError,
}

async fn can_have_role<'a>(
    ctx: &Context,
    role: &'a roles::CustomRole,
    member: &Member,
    server_id: GuildId,
) -> Result<(), Reason<'a>> {
    if let Some(duration) = &role.properties.time_requirement.map(Duration::from_std) {
        if let Ok(time_requirement) = duration {
            if let Some(time) = member.joined_at {
                let time_since_join = Utc::now().signed_duration_since(time);
                if &time_since_join < time_requirement {
                    return Err(NotEnoughTime(
                        Utc::now() + *time_requirement - time_since_join,
                    ));
                }
            }
        } else {
            println!("Duration too big to convert! Denying permission.");
            return Err(TimeConversionError);
        }
    }

    if let Some(incompatible_roles) = &role.properties.incompatible_roles {
        for role_name in incompatible_roles {
            if let Some(retrieved_role) =
                role_cache::get_role(ctx, server_id, role_name.clone()).await
            {
                if member.roles.contains(&retrieved_role.id) {
                    return Err(IncompatibleRole(role_name));
                }
            } else {
                println!(
                    "Could not retrieve \"{}\" role! Denying permission.",
                    role_name
                );
                return Err(RoleRetrievalError);
            }
        }
    }

    if let Some(required_roles) = &role.properties.required_roles {
        for role_name in required_roles {
            if let Some(retrieved_role) =
                role_cache::get_role(ctx, server_id, role_name.clone()).await
            {
                if !member.roles.contains(&retrieved_role.id) {
                    return Err(Reason::MissingRequiredRole(role_name));
                }
            } else {
                println!(
                    "Could not retrieve \"{}\" role! Denying permission.",
                    role_name
                );
                return Err(RoleRetrievalError);
            }
        }
    }

    Ok(())
}

async fn display_roles(ctx: &Context, msg: &Message, in_dms: bool) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(roles) = roles::get_role_list(ctx, server_id).await {
        let role_list = roles
            .iter()
            .filter_map(|aliases| aliases.get(0).map(|name| (name, aliases[1..].join(", "))))
            .fold(
                String::from("**Use `!role <role name>` to claim a role**"),
                |x, (name, aliases)| {
                    if aliases.is_empty() {
                        format!("{}\n{}", x, name)
                    } else {
                        format!("{}\n{} (*aliases:* {})", x, name, aliases)
                    }
                },
            );
        if in_dms {
            msg.author
                .direct_message(ctx, |m| {
                    m.embed(|e| {
                        e.title("Available roles");
                        e.description(role_list);
                        e
                    })
                })
                .await?;
        } else {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title("Available roles");
                        e.description(role_list);
                        e
                    })
                })
                .await?;
        }
    } else {
        println!(
            "Could not retrieve role list in {} (`{}`)",
            server_id.to_partial_guild(ctx).await?.name,
            server_id.0
        );
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(user_blacklist)]
#[sub_commands(add, delete, listroles, display, cache)]
pub async fn role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.delete(ctx).await.is_err() {
        warn!(ctx, msg);
    }
    if args.is_empty() || args.current().unwrap().to_lowercase().eq("list") {
        return display_roles(ctx, msg, true).await;
    }
    let role_name = format_role_name(args.rest());
    let server_id = msg.guild_id.ok_or(NotInGuild)?;

    if let Some(role) = role_cache::get_role(ctx, server_id, role_name).await {
        let mut member = server_id.member(ctx, msg.author.id).await?;
        let can_have_role = can_have_role(ctx, &role, &member, server_id).await;
        if can_have_role.is_ok()
            || msg.author.id == OWNER_ID
            || is_admin!(ctx, msg)
            || has_permission(
                ctx,
                server_id,
                msg.author.id,
                crate::constants::MANAGE_BOT_PERMS,
            )
            .await
        {
            if member.roles.contains(&role.id) {
                if member.remove_role(ctx, role.id).await.is_err() {
                    role_message!(
                        ctx,
                        msg,
                        role,
                        "The bot is missing the permissions to remove roles! Contact an admin."
                    );
                } else {
                    role_log!(
                        msg,
                        role,
                        "Role {role_name} ({role_id}) removed from {user_name} ({user_id})"
                    );
                    role_message!(
                        ctx,
                        msg,
                        role,
                        "The **{}** role has been removed from your profile",
                        role.name
                    );
                }
            } else if member.add_role(ctx, role.id).await.is_err() {
                role_message!(
                    ctx,
                    msg,
                    role,
                    "The bot is missing the permissions to give roles! Contact an admin."
                );
            } else {
                role_log!(
                    msg,
                    role,
                    "Role {role_name} ({role_id}) given to {user_name} ({user_id})"
                );
                role_message!(
                    ctx,
                    msg,
                    role,
                    "You have been given the **{}** role.",
                    role.name
                );
            }
        } else {
            match can_have_role.unwrap_err() {
                NotEnoughTime(date) => {
                    role_log!(msg, role,
                        "Time requirement not met for role {role_name} ({role_id}) to {user_name} ({user_id})."
                    );
                    role_message!(
                        ctx,
                        msg,
                        role,
                        "You have not been on the server for enough time to be able to claim the \
**{}** role! It will unlock on {}",
                        role.name,
                        date.format("%B %-d %Y at %R (UTC)")
                    );
                }
                IncompatibleRole(incompatible_role_name) => {
                    role_log!(
                        msg,
                        role,
                        "Incompatible role \"{}\" for role {role_name} ({role_id}) \
to {user_name} ({user_id})",
                        incompatible_role_name
                    );
                    role_message!(
                        ctx,
                        msg,
                        role,
                        "You have the **{}** role, which is incompatible with the role you \
are trying to claim.",
                        incompatible_role_name
                    );
                }
                MissingRequiredRole(missing_role_name) => {
                    role_log!(
                        msg,
                        role,
                        "Missing required role \"{}\" for giving {role_name} ({role_id}) to \
{user_name} ({user_id})",
                        missing_role_name
                    );
                    role_message!(
                        ctx,
                        msg,
                        role,
                        "You are missing the **{}** role, which is required for the role you \
are trying to claim.",
                        missing_role_name
                    );
                }
                other_error => println!(
                    "Error trying to claim role \"{}\" in {:?}: {:?}",
                    role.name, server_id, other_error
                ),
            }
        }
    } else {
        msg.author
            .dm(ctx, |m| {
                m.embed(|e| {
                    e.description("The role you are trying to claim does not exist on the server.")
                        .colour(serenity::utils::Colour::RED)
                })
            })
            .await?;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;
    if let Ok(role_id) = args.parse::<RoleId>() {
        if let Some(role) = server_id.roles(ctx).await?.get(&role_id) {
            match get_json_from_message::<roles::RoleProperties>(msg).await {
                Ok(role_properties) => {
                    role_cache::add_role(
                        ctx,
                        server_id,
                        roles::CustomRole {
                            id: role_id,
                            name: format_role_name(&role.name),
                            properties: role_properties,
                            colour: role.colour,
                        },
                    )
                    .await?;
                    println!("Created role {} on {}", role.name, server_id);
                    success!(ctx, msg);
                }
                Err(err) => handle_json_error!(ctx, msg, err),
            }
        } else {
            failure!(
                ctx,
                msg,
                "The role {} does not exist on the server.",
                role_id.mention()
            );
        }
    } else {
        failure!(ctx, msg, "The first argument must be a role mention!");
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(is_admin)]
#[aliases("remove")]
pub async fn delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let server_id = msg.guild_id.ok_or(NotInGuild)?;
    let role_name = format_role_name(args.rest());
    if let Some(role) = role_cache::get_role(ctx, server_id, role_name).await {
        role_cache::delete_role(ctx, server_id, role.id).await?;
        println!("Removed role {} on {}", role.name, server_id);
        success!(ctx, msg);
    } else {
        failure!(ctx, msg, "The first argument must be a role mention.");
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[aliases("roles")]
pub async fn listroles(ctx: &Context, msg: &Message) -> CommandResult {
    display_roles(ctx, msg, false).await
}

#[command]
#[only_in(guilds)]
#[checks(allowed_blacklist)]
#[aliases("show")]
pub async fn display(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        return display_roles(ctx, msg, false).await;
    }
    let server_id = msg.guild_id.ok_or(NotInGuild)?;
    let role_name = format_role_name(args.rest());
    if let Some(role) = role_cache::get_role(ctx, server_id, role_name).await {
        let aliases = roles::get_aliases(ctx, server_id, role.id).await;
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.colour(role.colour);
                    e.title(&role.name);
                    e.description(format!("Role ID: `{}`", &role.id.0));
                    if let Some(aliases) = aliases {
                        e.field("Aliases", aliases.join(", "), false);
                    }
                    if let Some(time_requirement) = role.properties.time_requirement {
                        e.field("Time requirement", format_duration(time_requirement), true);
                    }
                    if let Some(incompatible_roles) = &role.properties.incompatible_roles {
                        e.field("Incompatible roles", incompatible_roles.join(", "), false);
                    }
                    if let Some(required_roles) = &role.properties.required_roles {
                        e.field("Required roles", required_roles.join(", "), false);
                    }
                    e
                })
            })
            .await?;
    } else {
        failure!(ctx, msg, "No role by that name exists!");
    }
    Ok(())
}

#[command]
#[owners_only]
#[checks(is_admin)]
async fn cache(ctx: &Context) -> CommandResult {
    let role_cache = {
        let data_read = ctx.data.read().await;
        data_read.get::<role_cache::RoleCache>().unwrap().clone()
    };
    println!("=== ROLE CACHE ===\n{:?}\n=== END ===", role_cache);
    Ok(())
}
