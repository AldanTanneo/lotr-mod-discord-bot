use crate::constants::*;

use mysql_async::prelude::*;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::application::{
    component::ComponentType,
    interaction::message_component::{
        MessageComponentInteraction, MessageComponentInteractionData,
    },
    interaction::Interaction,
};
use serenity::model::prelude::*;
use serenity::utils::colours;

use crate::utils::InteractionEasyResponse;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::playing(
            "The Lord of the Rings Mod: Bringing Middle-earth to Minecraft",
        ))
        .await;

        if let Err(e) = OWNER_ID
            .to_user(&ctx)
            .await
            .unwrap()
            .dm(&ctx, |m| {
                m.embed(|e| {
                    e.title("Bot started and ready!")
                        .description(format!("Guilds: {}", ready.guilds.len()))
                        .footer(|f| f.text("Use !guilds to see all guilds"))
                        .colour(colours::branding::GREEN)
                })
            })
            .await
        {
            println!("Error starting the bot: {}", e);
        } else {
            println!("Started bot!");
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, guild: Option<Guild>) {
        if incomplete.unavailable {
            println!("Guild {} went offline", incomplete.id.0);
        } else {
            let guild_name: String = if let Some(guild) = guild {
                guild.name
            } else {
                let mut conn = crate::get_database_conn!(ctx);
                if let Ok(option) = conn
                    .query_first(format!(
                        "SELECT guild_name FROM {} WHERE guild_id = {}",
                        TABLE_LIST_GUILDS, incomplete.id.0
                    ))
                    .await
                {
                    option.unwrap_or_else(|| "unregistered guild".into())
                } else {
                    "unknown (database query failed)".into()
                }
            };

            OWNER_ID
                .to_user(&ctx)
                .await
                .unwrap()
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        e.title(format!(
                            "Bot was kicked from {} (`{}`)",
                            guild_name, incomplete.id.0
                        ))
                        .colour(colours::branding::RED)
                    })
                })
                .await
                .unwrap();
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let guild_owner = guild
                .owner_id
                .to_user(&ctx)
                .await
                .map_or_else(|_| "Unknown user".to_string(), |u| u.tag());

            if let Err(e) =
                crate::database::admin_data::add_admin(&ctx, guild.id, guild.owner_id, false).await
            {
                println!(
                    "=== ERROR ===\nCould not add Guild owner as admin: {}\n=== END ===",
                    e
                );
            }

            OWNER_ID
                .to_user(&ctx)
                .await
                .unwrap()
                .dm(&ctx, |m| {
                    m.embed(|e| {
                        if let Some(num_members) = guild.approximate_member_count {
                            e.description(format!(
                                "Owner: {}\n{} members",
                                guild_owner, num_members
                            ));
                        } else {
                            e.description(format!("Owner: {}", guild_owner));
                        }
                        if let Some(icon) = guild.icon_url() {
                            e.thumbnail(icon);
                        }
                        e.title(format!(
                            "Bot was added to {} (`{}`)",
                            guild.name, guild.id.0
                        ))
                        .colour(colours::branding::GREEN)
                    })
                })
                .await
                .unwrap();
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let guild_id = match reaction.guild_id {
            None => return,
            Some(guild_id) => {
                if guild_id != LOTR_DISCORD {
                    return;
                }
                guild_id
            }
        };

        if reaction.emoji.unicode_eq("❓") {
            crate::qa_answers::handle_reaction(&ctx, reaction, guild_id).await;
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        let guild_id = match message.guild_id {
            None => return,
            Some(guild_id) => {
                if guild_id != LOTR_DISCORD {
                    return;
                }
                guild_id
            }
        };

        if message.referenced_message.is_none() {
            return;
        }

        crate::qa_answers::handle_message(&ctx, &message, guild_id).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(
            component_interaction @ MessageComponentInteraction {
                user,
                data:
                    MessageComponentInteractionData {
                        component_type: ComponentType::Button,
                        custom_id,
                        ..
                    },
                ..
            },
        ) = &interaction
        {
            if let Some(bug_id) = custom_id
                .strip_prefix("bug_unsubscribe__")
                .and_then(|s| s.parse::<u64>().ok())
            {
                if crate::database::bug_reports::is_notified_user(&ctx, bug_id, user.id).await
                    != Some(true)
                {
                    component_interaction
                        .say_ephemeral(
                            &ctx,
                            format!(
                                ":x: You are not subscribed to bug LOTR-{}.

To see all your active notifications type  `!bug notifications`",
                                bug_id
                            ),
                        )
                        .await;
                } else if let Err(e) =
                    crate::database::bug_reports::remove_notified_user(&ctx, bug_id, user.id).await
                {
                    println!(
                        "=== ERROR ===\nCould not remove {} {:?} \
from LOTR-{} notifications\nError: {}\n=== END ===",
                        user.tag(),
                        user.id,
                        bug_id,
                        e
                    );
                } else {
                    component_interaction
                        .say_ephemeral(
                            &ctx,
                            format!(
                                "You have successfully been unsubscribed from bug LOTR-{}.

To see all your active notifications type  `!bug notifications`",
                                bug_id
                            ),
                        )
                        .await;
                }
            } else if let Some(bug_id) = custom_id
                .strip_prefix("bug_subscribe__")
                .and_then(|s| s.parse::<u64>().ok())
            {
                if crate::database::bug_reports::is_notified_user(&ctx, bug_id, user.id).await
                    != Some(false)
                {
                    component_interaction
                        .say_ephemeral(
                            &ctx,
                            format!(
                                ":x: You are already subscribed to bug LOTR-{}.

To see all your active notifications type  `!bug notifications`",
                                bug_id
                            ),
                        )
                        .await;
                } else if let Err(e) =
                    crate::database::bug_reports::add_notified_user(&ctx, bug_id, user.id).await
                {
                    println!(
                        "=== ERROR ===\nCould not add {} {:?} \
to LOTR-{} notifications\nError: {}\n=== END ===",
                        user.tag(),
                        user.id,
                        bug_id,
                        e
                    );
                } else {
                    component_interaction
                        .say_ephemeral(
                            &ctx,
                            format!(
                                "You have successfully been subscribed to bug LOTR-{}.

To see all your active notifications type  `!bug notifications`",
                                bug_id
                            ),
                        )
                        .await;
                }
            }
        }
    }
}
