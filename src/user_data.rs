use crate::constants::OWNER_ID;
use crate::mysql;
use crate::serenity;

struct ApiKeys {
    curseforge: String,
    google: String,
}

impl ApiKeys {
    fn new() -> Result<Self, std::env::VarError> {
        Ok(Self {
            curseforge: std::env::var("CURSEFORGE_API_KEY")?,
            google: std::env::var("GOOGLE_API_KEY")?,
        })
    }

    fn curseforge(&self) -> &str {
        &self.curseforge
    }
    fn google(&self) -> &str {
        &self.google
    }
}

pub struct Data {
    db_pool: mysql::MySqlPool,
    reqwest_client: reqwest::Client,
    api_keys: ApiKeys,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl Data {
    pub async fn new(
        ctx: &serenity::Context,
        ready: &serenity::Ready,
        framework: &poise::Framework<Data, Error>,
        db_uri: String,
    ) -> Result<Self> {
        let api_keys = ApiKeys::new()?;

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

        framework.options().commands.iter().for_each(|cmd| {
            println!("|{: ^20}|", cmd.name);
            if cmd.name == "online" {
                if let Some(slash_command) = cmd.create_as_slash_command() {
                    online_command_builder.add_application_command(slash_command);
                }
            } else {
                if let Some(slash_command) = cmd.create_as_slash_command() {
                    commands_builder.add_application_command(slash_command);
                }
                if let Some(context_menu_command) = cmd.create_as_context_menu_command() {
                    commands_builder.add_application_command(context_menu_command);
                }
            }
        });

        if let Ok(Ok(test_guild_id)) =
            std::env::var("TEST_SLASH_COMMANDS").map(|s| s.parse::<u64>())
        {
            let mut merged_commands = commands_builder.0;
            merged_commands.append(&mut online_command_builder.0);
            let commands_json = serde_json::Value::Array(merged_commands);
            println!("Testing environment...");
            ctx.http
                .create_guild_application_commands(test_guild_id, &commands_json)
                .await?;
            println!("Created commands");
        } else {
            let commands_json = serde_json::Value::Array(commands_builder.0);
            let online_command_json = serde_json::Value::Array(online_command_builder.0);

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
                    .colour(crate::serenity::colours::branding::GREEN)
                })
            })
            .await?;

        Ok(Self {
            db_pool,
            reqwest_client,
            api_keys,
        })
    }

    pub fn db_pool(&self) -> &mysql::MySqlPool {
        &self.db_pool
    }

    pub fn reqwest_client(&self) -> &reqwest::Client {
        &self.reqwest_client
    }

    pub fn curseforge_api_key(&self) -> &str {
        self.api_keys.curseforge()
    }

    pub fn google_api_key(&self) -> &str {
        self.api_keys.google()
    }
}
