use poise::{ApplicationCommandTree, SlashCommand, SlashCommandMeta};

use crate::constants::{discord_colours, OWNER_ID};
use crate::mysql;
use crate::serenity;

pub struct Data {
    pub db_pool: mysql::MySqlPool,
    pub reqwest_client: reqwest::Client,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T = ()> = std::result::Result<T, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl Data {
    pub async fn new(
        ctx: &serenity::Context,
        ready: &serenity::Ready,
        framework: &poise::Framework<Data, Error>,
        db_uri: String,
    ) -> Result<Self> {
        ctx.set_activity(serenity::Activity::playing(
            "The Lord of the Rings Mod: Bringing Middle-earth to Minecraft",
        ))
        .await;

        let db_pool = mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&db_uri)
            .await?;

        let minecraft_servers = sqlx::query_as::<_, (u64,)>("SELECT server_id FROM mc_server_ip")
            .fetch_all(&db_pool)
            .await?;

        println!("MC servers: {}", minecraft_servers.len());

        let mut commands_builder = serenity::CreateApplicationCommands::default();
        let mut online_command_builder = serenity::CreateApplicationCommands::default();
        framework
            .options()
            .application_options
            .commands
            .iter()
            .for_each(|cmd| match cmd {
                ApplicationCommandTree::Slash(SlashCommandMeta::Command(SlashCommand {
                    name: "online",
                    ..
                })) => {
                    online_command_builder.create_application_command(|f| cmd.create(f));
                }
                _ => {
                    commands_builder.create_application_command(|f| cmd.create(f));
                }
            });

        let commands_json = serde_json::Value::Array(commands_builder.0);
        let online_command_json = serde_json::Value::Array(online_command_builder.0);

        if let Ok(Ok(test_guild_id)) =
            std::env::var("TEST_SLASH_COMMANDS").map(|s| s.parse::<u64>())
        {
            ctx.http
                .create_guild_application_commands(test_guild_id, &commands_json)
                .await?;
            ctx.http
                .create_guild_application_commands(test_guild_id, &online_command_json)
                .await?;
        } else {
            ctx.http
                .create_global_application_commands(&commands_json)
                .await?;
            for guild_id in minecraft_servers {
                if let Err(e) = ctx
                    .http
                    .create_guild_application_commands(guild_id.0, &online_command_json)
                    .await
                {
                    println!(
                        "Error registering the /online command on guild {}: {}",
                        guild_id.0, e
                    );
                }
            }
        }

        let reqwest_client = reqwest::Client::builder().use_rustls_tls().build()?;

        OWNER_ID
            .to_user(ctx)
            .await?
            .dm(ctx, |m| {
                m.embed(|e| {
                    e.title(format!(
                        "Bot started and ready!\n\tGuilds: {}\n\t_Do `!guilds` to see all guilds_",
                        ready.guilds.len(),
                    ))
                    .colour(discord_colours::GREEN)
                })
            })
            .await?;

        Ok(Self {
            db_pool,
            reqwest_client,
        })
    }
}
