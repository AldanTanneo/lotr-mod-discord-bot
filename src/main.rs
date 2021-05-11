//! Discord bot used in the [LOTR Mod Community Discord](https://discord.gg/QXkZzKU)
//! and a few other servers.
//!
//! Includes LOTR Mod-related commands, a small [permissions system][commands::admin],
//! [custom commands][commands::custom_commands], and useful commands for
//! technical support or easy reference.
//!
//! For a list of commands, see [here][commands]

pub mod announcement;
pub mod api;
pub mod check;
pub mod commands;
pub mod constants;
pub mod database;
pub mod utils;

use mysql_async::{
    prelude::Queryable,
    {OptsBuilder, Pool},
};
use serenity::async_trait;
use serenity::client::{ClientBuilder, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::http::client::Http;
use serenity::model::prelude::*;
use std::{env, sync::Arc};

use api::ReqwestClient;
use check::{after_hook, dispatch_error_hook};
use commands::{
    admin::*, announcements::*, bug_reports::*, custom_commands::*, general::*, help::*, meme::*,
    servers::*, wiki::*,
};
use constants::{BOT_ID, OWNER_ID, TABLE_LIST_GUILDS};
use database::{config::get_prefix, maintenance::update_list_guilds, DatabasePool};

#[group]
#[default_command(custom_command)]
#[commands(
    custom_command,
    help,
    renewed,
    curseforge,
    prefix,
    forge,
    coremod,
    invite,
    server_ip,
    online,
    donate,
    facebook,
    discord
)]
struct General;

#[group]
#[commands(floppa, aeugh, dagohon)]
struct Meme;

#[group]
#[commands(wiki, tolkien, minecraft)]
struct Wiki;

#[group]
#[commands(track, buglist, bug, resolve)]
struct BugReports;

#[group]
#[commands(admin, floppadd, blacklist, announce, floppadmin, listguilds, define)]
struct Moderation;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::playing(
            "The Lord of the Rings Mod: Bringing Middle-earth to Minecraft",
        ))
        .await;

        OWNER_ID
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
            .unwrap();

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

#[tokio::main]
async fn main() {
    // get environment variables for bot login
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_name: String = env::var("DB_NAME").expect("Expected an environment variable DB_NAME");
    let db_user: String = env::var("DB_USER").expect("Expected an environment variable DB_USER");
    let db_password: String =
        env::var("DB_PASSWORD").expect("Expected an environment variable DB_PASSWORD");
    let db_server: String =
        env::var("DB_SERVER").expect("Expected an environment variable DB_SERVER");
    let db_port: u16 = env::var("DB_PORT")
        .expect("Expected an environment variable DB_PORT")
        .parse()
        .unwrap();

    // create database pool for bot guild data
    let pool: Pool = Pool::new(
        OptsBuilder::default()
            .user(Some(db_user))
            .db_name(Some(db_name))
            .ip_or_hostname(db_server)
            .pass(Some(db_password))
            .tcp_port(db_port),
    );

    // reqwest client for API calls
    let reqwest_client = Arc::new(
        reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .expect("Could not build the reqwest client"),
    );
    let cloned_client = Arc::clone(&reqwest_client);

    // initialize bot framework
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("") // remove default prefix and
                // add dynamic prefix defaulting to "!"
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move {
                        Some(
                            get_prefix(ctx, msg.guild_id)
                                .await
                                .unwrap_or_else(|| "!".into()),
                        )
                    })
                })
                // bot reacts on mention
                .on_mention(Some(BOT_ID))
                // sole owner is constants::OWNER_ID
                .owners(vec![OWNER_ID].into_iter().collect())
                // "wiki", "Wiki", "wIKi" are all valid commands
                .case_insensitivity(true)
                // supports multiline commands
                .delimiters(vec![' ', '\n'])
        })
        // failed checks handler
        .on_dispatch_error(dispatch_error_hook)
        .after(after_hook)
        // command groups
        .group(&MEME_GROUP)
        .group(&WIKI_GROUP)
        .group(&MODERATION_GROUP)
        .group(&BUGREPORTS_GROUP)
        .group(&GENERAL_GROUP)
        // rate limiting some commands
        .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        .await;

    // building client
    let mut client =
        ClientBuilder::new_with_http(Http::new(reqwest_client, &format!("Bot {}", token)))
            .event_handler(Handler)
            .framework(framework)
            .type_map_insert::<DatabasePool>(Arc::new(pool))
            .type_map_insert::<ReqwestClient>(cloned_client)
            .await
            .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        // basic error logging
        println!("An error occurred while running the client: {:?}", why);
    }
}
