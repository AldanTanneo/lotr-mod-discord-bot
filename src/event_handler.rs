use crate::constants::*;
use crate::database::{maintenance::update_list_guilds, DatabasePool};

use mysql_async::prelude::*;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::*;

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
                m.content(format!(
                    "Bot started and ready!\n\tGuilds: {}\n\t_Do `!guilds` to see all guilds_",
                    ready.guilds.len(),
                ))
            })
            .await
        {
            println!("Error starting the bot: {:?}", e);
        }

        println!("UPDATING GUILD LIST");
        match update_list_guilds(&ctx).await {
            Ok(n) => println!(
                "Successfully updated list_guilds table, before - after = {}",
                n
            ),
            Err(e) => println!("Error updating list_guilds table: {:?}", e),
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _: Option<Guild>) {
        if !incomplete.unavailable {
            let pool = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<DatabasePool>()
                    .expect("Could not retrieve database pool")
                    .clone()
            };
            let guild_name: Option<String> = if let Ok(mut conn) = pool.get_conn().await {
                if let Ok(option) = conn
                    .query_first(format!(
                        "SELECT guild_name FROM {} WHERE guild_id = {}",
                        TABLE_LIST_GUILDS, incomplete.id.0
                    ))
                    .await
                {
                    option
                } else {
                    Some("unknown (database query failed)".into())
                }
            } else {
                Some("unknown (database connection failed)".into())
            };
            OWNER_ID
                .to_user(&ctx)
                .await
                .unwrap()
                .dm(&ctx, |m| {
                    m.content(format!(
                        "Bot was kicked from {} (`{}`)",
                        guild_name.unwrap_or_else(|| "unregistered guild".into()),
                        incomplete.id.0
                    ))
                })
                .await
                .unwrap();
        } else {
            println!("Guild {} went offline", incomplete.id.0);
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            OWNER_ID
                .to_user(&ctx)
                .await
                .unwrap()
                .dm(&ctx, |m| {
                    m.content(format!(
                        "Bot was added to {} (`{}`)",
                        guild.name, guild.id.0
                    ))
                })
                .await
                .unwrap();
        }
    }
}