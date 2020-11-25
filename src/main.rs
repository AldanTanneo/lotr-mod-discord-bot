use itertools::free::join;
use mysql_async::prelude::*;
use mysql_async::*;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::prelude::ReactionType;
use serenity::model::{
    channel::Message,
    gateway::{Activity, Ready},
    id::{GuildId, UserId},
};
use serenity::prelude::TypeMapKey;
use std::{env, sync::Arc};

const BOT_ID: UserId = UserId(780858391383638057);
const OWNER_ID: UserId = UserId(405421991777009678);

const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";

#[derive(Debug, PartialEq, Eq)]
struct ServerPrefix {
    server_id: u64,
    prefix: Option<String>,
}

pub async fn get_prefix(ctx: &Context, guild_id: Option<GuildId>) -> String {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool
        .get_conn()
        .await
        .expect("Could not connect to database");
    let server_id: u64 = if let Some(id) = guild_id {
        id.into()
    } else {
        0
    };
    let res = conn
        .query_first(format!(
            "SELECT prefix FROM {} WHERE server_id={}",
            TABLE_PREFIX, server_id
        ))
        .await;
    if let Ok(Some(prefix)) = res {
        prefix
    } else {
        set_prefix(ctx, guild_id, "!", false).await.unwrap();
        "!".to_string()
    }
}

pub async fn set_prefix(
    ctx: &Context,
    guild_id: Option<GuildId>,
    prefix: &str,
    update: bool,
) -> Result<()> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<DatabasePool>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let mut conn = pool
        .get_conn()
        .await
        .expect("Could not connect to database");
    let server_id: u64 = if let Some(id) = guild_id {
        id.into()
    } else {
        0
    };
    let req = if update {
        format!(
            "UPDATE {} SET prefix = :prefix WHERE server_id = :server_id",
            TABLE_PREFIX
        )
    } else {
        format!(
            "INSERT INTO {} (server_id, prefix) VALUES (:server_id, :prefix)",
            TABLE_PREFIX
        )
    };
    conn.exec_batch(
        req.as_str(),
        vec![ServerPrefix {
            server_id,
            prefix: Some(prefix.to_string()),
        }]
        .iter()
        .map(|p| {
            params! {
                "server_id" => p.server_id,
                "prefix" => &p.prefix,
            }
        }),
    )
    .await?;
    Ok(())
}

struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}

#[group]
#[commands(renewed, help, prefix, tos, curseforge)]
struct General;

#[group]
#[default_command(wiki)]
#[commands(user, category, template, search, random)]
struct Wiki;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let game =
            Activity::playing("The Lord of the Rings Mod: Bringing Middle-earth to Minecraft");
        ctx.set_activity(game).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_user_id(BOT_ID) {
            let prefix = get_prefix(&ctx, msg.guild_id).await;
            msg.channel_id
                .send_message(ctx, |m| {
                    m.content(format!("My prefix here is \"{}\"", prefix))
                })
                .await
                .expect("Failed to send message");
        }
    }
}

#[tokio::main]
async fn main() {
    let db_name: String = env::var("DB_NAME").expect("Expected an environment variable DB_NAME");
    let db_userdb_password: String =
        env::var("DB_USER").expect("Expected an environment variable DB_USER");
    let db_password: String =
        env::var("DB_PASSWORD").expect("Expected an environment variable DB_PASSWORD");
    let db_server: String =
        env::var("DB_SERVER").expect("Expected an environment variable DB_SERVER");
    let db_portdb_server: u16 = env::var("DB_PORT")
        .expect("Expected an environment variable DB_PORT")
        .parse()
        .unwrap();

    let pool: Pool = Pool::new(
        OptsBuilder::default()
            .user(Some(db_userdb_password))
            .db_name(Some(db_name))
            .ip_or_hostname(db_server)
            .pass(Some(db_password))
            .tcp_port(db_portdb_server),
    );

    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(|ctx, msg| {
                Box::pin(async move { Some(get_prefix(ctx, msg.guild_id).await) })
            })
            .allow_dm(false)
            .on_mention(Some(BOT_ID))
            .owners(vec![OWNER_ID].into_iter().collect())
        })
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;

        data.insert::<DatabasePool>(Arc::new(pool));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn renewed(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Use the 1.7.10 version");
                e.description(
                    "The 1.15.2 version of the mod is a work in progress, missing many features.
You can find those in the full 1.7.10 Legacy edition [here](https://lotrminecraftmod.fandom.com/wiki/Template:Main_Version)",
                );
                e
            });

            m
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let prefix = get_prefix(ctx, msg.guild_id).await;
    msg.author
        .direct_message(ctx, |m| {
            m.content(format!("My prefix here is \"{}\"", prefix));
            m.embed(|e| {
                e.title("Available commands");
                e.field("General commands", "`renewed`, `tos`, `curseforge`, `help`, `prefix`", false);
                e.field("Wiki commands", "`wiki`, `wiki user`, `wiki category`, `wiki template`, `wiki search`, `wiki random`", false);
                e
            });
            m
        })
        .await?;

    msg.react(ctx, ReactionType::from('✅')).await?;

    Ok(())
}

fn wiki_url(args: Args, caps: bool, del: &str) -> String {
    join(
        args.rest().split_whitespace().map(|word| {
            if caps {
                let (a, b) = word.split_at(1);
                format!("{}{}", a.to_uppercase(), b)
            } else {
                word.to_string()
            }
        }),
        del,
    )
}

#[command]
#[required_permissions("ADMINISTRATOR")]
#[owner_privilege(true)]
#[max_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let prefix = get_prefix(ctx, msg.guild_id).await;
        msg.channel_id
            .send_message(ctx, |m| {
                m.content(format!("My prefix here is \"{}\"", prefix))
            })
            .await?;
    } else {
        let new_prefix = args.single::<String>();
        if let Ok(p) = new_prefix {
            if set_prefix(ctx, msg.guild_id, &p, true).await.is_ok() {
                msg.channel_id
                    .send_message(ctx, |m| {
                        m.content(format!("Set the new prefix to \"{}\"", p))
                    })
                    .await?;
            } else {
                msg.channel_id
                    .send_message(ctx, |m| m.content("Failed to set the new prefix!"))
                    .await?;
            }
        } else {
            msg.channel_id
                .send_message(ctx, |m| m.content("Invalid new prefix!"))
                .await?;
        }
    }
    Ok(())
}

#[command]
async fn tos(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(
            "This is the Discord server of the **Lord of the Rings Mod**, not the official Minecraft server of the mod.
Their Discord can be found here: https://discord.gg/gMNKaX6",
        )
        })
        .await?;
    msg.delete(ctx).await?;
    Ok(())
}

#[command]
async fn curseforge(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| m.embed(|e|{
        e.title("Link to the Renewed version");
        e.description("The Renewed edition of the mod can be found on [Curseforge](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-renewed)");
        e
    })).await?;
    Ok(())
}

// -------------- WIKI COMMANDS -------------

#[command]
#[aliases("page")]
async fn wiki(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "https://lotrminecraftmod.fandom.com/wiki/{}",
                wiki_url(args, true, "_")
            ))
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}

#[command]
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "https://lotrminecraftmod.fandom.com/wiki/Special:Search?query={}&scope=internal&navigationSearch=true",
                wiki_url(args, false, "+")
            ))
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}
#[command]
async fn user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "https://lotrminecraftmod.fandom.com/wiki/User:{}",
                wiki_url(args, false, "_")
            ))
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}
#[command]
async fn category(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "https://lotrminecraftmod.fandom.com/wiki/Category:{}",
                wiki_url(args, true, "_")
            ))
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}
#[command]
async fn template(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "https://lotrminecraftmod.fandom.com/wiki/Template:{}",
                wiki_url(args, false, "_")
            ))
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.content("https://lotrminecraftmod.fandom.com/wiki/Special:Random")
        })
        .await?;
    Ok(())
}
