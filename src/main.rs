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
pub mod role_cache;
pub mod utils;

use dashmap::DashMap;
use mysql_async::{OptsBuilder, Pool};
use serenity::client::ClientBuilder;
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::http::client::Http;
use std::{env, sync::Arc};

use api::ReqwestClient;
use check::{after_hook, dispatch_error_hook};
use commands::{
    admin::*, announcements::*, bug_reports::*, custom_commands::*, general::*, help::*, meme::*,
    roles::*, servers::*, wiki::*,
};
use constants::{BOT_ID, OWNER_ID};
use database::{
    config::{get_prefix, PrefixCache},
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

#[group]
#[commands(custom_command)]
#[default_command(custom_command)]
struct CustomCommand;

#[tokio::main]
async fn main() {
    // get environment variables for bot login
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("APPLICATION_ID must be a valid u64");

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

    let role_cache_map = DashMap::new();
    let prefix_cache = DashMap::new();

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
        // Must go last
        .group(&CUSTOMCOMMAND_GROUP)
        // rate limiting some commands
        .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        .await;

    let mut http = Http::new(reqwest_client, &format!("Bot {}", &token));
    http.application_id = application_id;
    // building client
    let mut client = ClientBuilder::new_with_http(http)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<DatabasePool>(Arc::new(pool))
        .type_map_insert::<ReqwestClient>(cloned_client)
        .type_map_insert::<RoleCache>(Arc::new(role_cache_map))
        .type_map_insert::<PrefixCache>(Arc::new(prefix_cache))
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        // basic error logging
        println!("An error occurred while running the client: {:?}", why);
    }
}
