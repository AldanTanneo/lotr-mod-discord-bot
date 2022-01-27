use crate::constants::*;

use mysql_async::prelude::*;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::*;
use serenity::utils::colours;

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
        if !incomplete.unavailable {
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
        } else {
            println!("Guild {} went offline", incomplete.id.0);
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let guild_owner = guild
                .owner_id
                .to_user(&ctx)
                .await
                .map(|u| u.tag())
                .unwrap_or_else(|_| "Unknown user".to_string());

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
}
