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
pub mod event_handler;
pub mod qa_answers;
pub mod role_cache;
pub mod utils;

use mysql_async::OptsBuilder;
use serenity::client::ClientBuilder;
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::http::client::Http;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;

use api::ReqwestClient;
use check::{after_hook, dispatch_error_hook};
use commands::{
    admin::*, announcements::*, bug_reports::*, custom_commands::*, general::*, help::*, meme::*,
    qa_setup::*, roles::*, servers::*, wiki::*,
};
use constants::{BOT_ID, OWNER_ID};
use database::{
    config::{get_prefix, PrefixCache},
    qa_data::QaChannelsCache,
    DatabasePool,
};
use event_handler::Handler;
use role_cache::RoleCache;

#[group]
#[commands(
    help, renewed, curseforge, prefix, forge, coremod, invite, server_ip, online, donate, facebook,
    discord, user_info, role, listroles, instagram
)]
struct General;

#[group]
#[commands(
    qa_moderator,
    qa_answer_channel,
    qa_question_channel,
    qa_disable,
    qa_summary,
    qa_cache
)]
#[prefix("q&a")]
#[default_command(qa_summary)]
struct QA;

#[group]
#[commands(floppa, aeugh, dagohon, colour)]
struct Meme;

#[group]
#[commands(wiki, tolkien, minecraft)]
struct Wiki;

#[group]
#[commands(track, buglist, bug, resolve)]
struct BugReports;

#[group]
#[commands(
    admin, floppadd, blacklist, announce, floppadmin, listguilds, define, shutdown
)]
struct Moderation;

#[group]
#[commands(custom_command)]
#[default_command(custom_command)]
struct CustomCommand;

#[derive(Clone)]
pub struct FrameworkKey(Arc<StandardFramework>);

impl TypeMapKey for FrameworkKey {
    type Value = Self;
}

impl std::ops::Deref for FrameworkKey {
    type Target = StandardFramework;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl FrameworkKey {
    pub fn new(framework: StandardFramework) -> Self {
        Self(Arc::new(framework))
    }
    pub fn as_arc(&self) -> Arc<StandardFramework> {
        self.0.clone()
    }
}

#[tokio::main]
async fn main() {
    // get environment variables for bot login
    // Discord token & application id
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("APPLICATION_ID must be a valid u64");

    // Database credentials
    let db_name: String = env::var("DB_NAME").expect("Expected an environment variable DB_NAME");
    let db_user: String = env::var("DB_USER").expect("Expected an environment variable DB_USER");
    let db_password: String =
        env::var("DB_PASSWORD").expect("Expected an environment variable DB_PASSWORD");
    let db_server: String =
        env::var("DB_SERVER").expect("Expected an environment variable DB_SERVER");
    let db_port: u16 = env::var("DB_PORT")
        .expect("Expected an environment variable DB_PORT")
        .parse()
        .expect("DB_PORT must be a valid u16");

    // create database pool for bot guild data
    let pool = DatabasePool::new(
        OptsBuilder::default()
            .user(Some(db_user))
            .db_name(Some(db_name))
            .ip_or_hostname(db_server)
            .pass(Some(db_password))
            .tcp_port(db_port),
    );

    // reqwest client for API calls
    let reqwest_client = ReqwestClient::new();

    let role_cache = RoleCache::new();
    let prefix_cache = PrefixCache::new();
    let qa_channels_cache = QaChannelsCache::new();

    // initialize bot framework
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("") // remove default prefix and
                // add dynamic prefix defaulting to "!"
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move { get_prefix(ctx, msg.guild_id.unwrap_or_default()).await })
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
        .group(&QA_GROUP)
        // Must go last
        .group(&CUSTOMCOMMAND_GROUP)
        // rate limiting some commands
        .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        .await;

    let mut http = Http::new(reqwest_client.as_arc(), &format!("Bot {}", &token));
    http.application_id = application_id;

    let framework = FrameworkKey::new(framework);

    // building client
    let mut client = ClientBuilder::new_with_http(http)
        .event_handler(Handler)
        .framework_arc(framework.as_arc())
        .type_map_insert::<DatabasePool>(pool)
        .type_map_insert::<ReqwestClient>(reqwest_client)
        .type_map_insert::<RoleCache>(role_cache)
        .type_map_insert::<PrefixCache>(prefix_cache)
        .type_map_insert::<QaChannelsCache>(qa_channels_cache)
        .type_map_insert::<FrameworkKey>(framework)
        .await
        .expect("Error creating client");

    {
        // Ctrl+C listener

        let shard_manager = client.shard_manager.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!("Shutting down...");
            shard_manager.clone().lock().await.shutdown_all().await;
        });
    }

    #[cfg(unix)]
    {
        // Sigterm listener

        let shard_manager = client.shard_manager.clone();
        tokio::spawn(async move {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap()
                .recv()
                .await
                .unwrap();
            println!("Shutting down...");
            shard_manager.lock().await.shutdown_all().await;
        });
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        // basic error logging
        println!("An error occurred while running the client: {:?}", why);
    }
}
