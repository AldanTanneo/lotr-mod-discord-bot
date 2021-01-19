mod announcement;
mod api;
mod check;
mod commands;
mod constants;
mod database;

use mysql_async::*;
use reqwest::redirect;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::futures::future::join;
use serenity::model::gateway::{Activity, Ready};
use std::{env, sync::Arc};

use api::structures::ReqwestClient;
use check::dispatch_error_hook;
use database::{config::get_prefix, DatabasePool};

use constants::*;

use commands::{admin::*, general::*, help::*, meme::*, servers::*, wiki::*};

#[group]
#[commands(
    help, renewed, tos, curseforge, prefix, forge, coremod, invite, server_ip, online
)]
struct General;

#[group]
#[commands(floppa, aeugh, dagohon)]
struct Meme;

#[group]
#[default_command(wiki)]
#[prefixes("wiki")]
#[commands(user, category, template, file, random, tolkien, minecraft)]
struct Wiki;

#[group]
#[only_in(guilds)]
#[prefixes("admin")]
#[default_command(list)]
#[commands(add, remove, list)]
struct Admin;

#[group]
#[commands(floppadd, blacklist, announce, floppadmin, listguilds)]
struct Moderation;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let (_, owner) = join(
            ctx.set_activity(Activity::playing(
                "The Lord of the Rings Mod: Bringing Middle-earth to Minecraft",
            )),
            OWNER_ID.to_user(&ctx),
        )
        .await;

        owner
            .unwrap()
            .dm(ctx, |m| {
                m.content(format!(
                    "Bot started and ready!\n\nGuilds: {}",
                    ready.guilds.len()
                ))
            })
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
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

    let pool: Pool = Pool::new(
        OptsBuilder::default()
            .user(Some(db_user))
            .db_name(Some(db_name))
            .ip_or_hostname(db_server)
            .pass(Some(db_password))
            .tcp_port(db_port),
    );

    let custom_redirect_policy = redirect::Policy::custom(|attempt| {
        if attempt.previous().len() > 5 {
            attempt.error("too many redirects")
        } else if attempt.url().host_str() != Some(WIKI_DOMAIN) {
            // prevent redirects outside of WIKI_DOMAIN
            attempt.stop()
        } else {
            attempt.follow()
        }
    });

    let request_client = reqwest::Client::builder()
        .redirect(custom_redirect_policy)
        .build()
        .expect("Could not build the reqwest client");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("")
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move {
                        Some(
                            get_prefix(ctx, msg.guild_id)
                                .await
                                .unwrap_or_else(|| "!".into()),
                        )
                    })
                })
                .on_mention(Some(BOT_ID))
                .owners(vec![OWNER_ID].into_iter().collect())
                .case_insensitivity(true)
                .delimiters(vec![' ', '\n'])
        })
        .on_dispatch_error(dispatch_error_hook)
        .group(&GENERAL_GROUP)
        .group(&MEME_GROUP)
        .group(&WIKI_GROUP)
        .group(&ADMIN_GROUP)
        .group(&MODERATION_GROUP)
        .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        .await;

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<DatabasePool>(Arc::new(pool));
        data.insert::<ReqwestClient>(Arc::new(request_client));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
