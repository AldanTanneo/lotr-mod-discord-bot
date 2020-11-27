mod database;
mod fandom;

use itertools::free::join;
use mysql_async::*;
use reqwest::redirect;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::{
    channel::Message,
    gateway::{Activity, Ready},
    id::{GuildId, UserId},
    prelude::ReactionType,
};
use std::{env, sync::Arc};

use database::{
    add_admin, add_floppa, get_admins, get_floppa, get_prefix, remove_admin, set_prefix,
    DatabasePool,
};
use fandom::{google_titles, GenericPage, Namespace, ReqwestClient, Wikis};

const BOT_ID: UserId = UserId(780858391383638057);
const OWNER_ID: UserId = UserId(405421991777009678);
const LOTR_DISCORD: GuildId = GuildId(405091134327619587);
const WIKI_DOMAIN: &str = "lotrminecraftmod.fandom.com";

#[group]
#[commands(help, renewed, tos, curseforge, prefix, floppa)]
struct General;

#[group]
#[default_command(wiki)]
#[prefixes("wiki")]
#[commands(user, category, template, file, random, tolkien)]
struct Wiki;

#[group]
#[only_in(guilds)]
#[prefixes("admin")]
#[default_command(list)]
#[commands(add, remove, list, floppadd)]
struct Admin;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let game =
            Activity::playing("The Lord of the Rings Mod: Bringing Middle-earth to Minecraft");
        ctx.set_activity(game).await;
    }
}

#[tokio::main]
async fn main() {
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

    let fandom_client = reqwest::Client::builder()
        .redirect(custom_redirect_policy)
        .build()
        .expect("Could not build the reqwest client");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("")
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move { Some(get_prefix(ctx, msg.guild_id).await) })
                })
                .allow_dm(false)
                .on_mention(Some(BOT_ID))
                .owners(vec![OWNER_ID].into_iter().collect())
        })
        .group(&GENERAL_GROUP)
        .group(&WIKI_GROUP)
        .group(&ADMIN_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;

        data.insert::<DatabasePool>(Arc::new(pool));
        data.insert::<ReqwestClient>(Arc::new(fandom_client));
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
                )
            });

            m
        })
        .await?;

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
                e.field(
                    "General commands",
                    "`renewed`, `tos`, `curseforge`, `help`",
                    false,
                );
                e.field(
                    "Wiki commands",
                    "`wiki`, `wiki user`, `wiki category`, `wiki template`, `wiki random`, `wiki tolkien`",
                    false,
                );
                e.field(
                    "Admin commands",
                    "`prefix`, `admin add`, `admin remove`, `admin list`",
                    false,
                );
                e
            });
            m
        })
        .await?;

    msg.react(ctx, ReactionType::from('✅')).await?;

    Ok(())
}

#[command]
async fn tos(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx,
            "This is the Discord server of the **Lord of the Rings Mod**, not the official Minecraft server of the mod.
Their Discord can be found here: https://discord.gg/gMNKaX6",
        )
        .await?;
    Ok(())
}

#[command]
async fn curseforge(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| m.embed(|e|{
        e.title("Link to the Renewed version");
        e.description("The Renewed edition of the mod can be found on [Curseforge](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-renewed)")
    })).await?;
    Ok(())
}

#[command]
#[max_args(1)]
async fn floppa(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let n = args.single::<u32>().ok();
    let url = if let Some(url) = get_floppa(ctx, n).await {
        url
    } else {
        "https://i.kym-cdn.com/photos/images/original/001/878/839/c6f.jpeg".to_string()
    };
    msg.channel_id.say(ctx, url).await?;
    Ok(())
}

// --------------------- Wiki Commands -------------------------

async fn wiki_search(
    ctx: &Context,
    msg: &Message,
    args: Args,
    namespace: Namespace,
    wiki: &Wikis,
) -> CommandResult {
    let srsearch = args.rest();
    let p = fandom::search(ctx, &namespace, srsearch, wiki).await;
    if let Some(page) = p {
        fandom::display(ctx, msg, &page, wiki).await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                format!("Couldn't find a {} for the given name!", namespace),
            )
            .await?;
    }
    Ok(())
}

#[command]
async fn wiki(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    if args.is_empty() {
        fandom::display(
            ctx,
            msg,
            &GenericPage {
                id: 331703,
                title: "The Lord of the Rings Minecraft Mod Wiki".into(),
            },
            &Wikis::LOTRMod,
        )
        .await?;
        return Ok(());
    }
    wiki_search(ctx, msg, args, Namespace::Page, wiki).await?;
    Ok(())
}

#[command]
async fn user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    wiki_search(ctx, msg, args, Namespace::User, wiki).await?;
    Ok(())
}

#[command]
async fn category(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    wiki_search(ctx, msg, args, Namespace::Category, wiki).await?;
    Ok(())
}
#[command]
async fn template(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    wiki_search(ctx, msg, args, Namespace::Template, wiki).await?;
    Ok(())
}

#[command]
async fn file(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    wiki_search(ctx, msg, args, Namespace::File, wiki).await?;
    Ok(())
}

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::LOTRMod;
    let p = fandom::random(ctx, wiki).await;
    if let Some(page) = p {
        fandom::display(ctx, msg, &page, wiki).await?;
    } else {
        msg.channel_id.say(ctx, "Couldn't execute query!").await?;
    }
    Ok(())
}

#[command]
async fn tolkien(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.rest();
    let result = google_titles(query, Wikis::TolkienGateway)
        .await
        .unwrap_or_else(|| "Main Page".into());
    let title = if let Some(title) = result.split(" - ").into_iter().next() {
        title
    } else {
        msg.channel_id
            .say(ctx, "Could not find a page with the given query!")
            .await?;
        return Ok(());
    };
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&title);
                e.url(format!(
                    "http://www.tolkiengateway.net/wiki/{}",
                    join(title.split_whitespace(), "_")
                ));
                e.author(|a| {
                    a.name("Tolkien Gateway");
                    a.url("http://www.tolkiengateway.net");
                    a.icon_url("https://i.ibb.co/VYKWK7V/favicon.png")
                })
            })
        })
        .await?;
    Ok(())
}

// ---------------- ADMIN COMMANDS -------------

#[command]
#[owner_privilege(true)]
#[max_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let prefix = get_prefix(ctx, msg.guild_id).await;
        msg.channel_id
            .say(ctx, format!("My prefix here is \"{}\"", prefix))
            .await?;
    } else {
        let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_default();
        if admins.contains(&msg.author.id) || msg.author.id == OWNER_ID {
            let new_prefix = args.single::<String>();
            if let Ok(p) = new_prefix {
                if !p.contains("<@") && set_prefix(ctx, msg.guild_id, &p, true).await.is_ok() {
                    msg.channel_id
                        .say(ctx, format!("Set the new prefix to \"{}\"", p))
                        .await?;
                } else {
                    msg.channel_id
                        .say(ctx, "Failed to set the new prefix!")
                        .await?;
                }
            } else {
                msg.channel_id.say(ctx, "Invalid new prefix!").await?;
            }
        } else {
            msg.channel_id
                .say(ctx, "You are not an admin on this server!")
                .await?;
            msg.react(ctx, ReactionType::from('❌')).await?;
        }
    }
    Ok(())
}

#[command]
#[max_args(1)]
#[min_args(1)]
async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_default();
    if (admins.contains(&msg.author.id) || msg.author.id == OWNER_ID) && !msg.mentions.is_empty() {
        if let Some(user) = msg
            .mentions
            .iter()
            .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
        {
            if !admins.contains(&user.id) {
                add_admin(ctx, msg.guild_id, user.id).await?;
            } else {
                msg.channel_id
                    .say(ctx, "This user is already a bot admin on this server!")
                    .await?;
            }
        } else {
            msg.channel_id
                .say(
                    ctx,
                    "Mention a user you wish to promote to bot admin for this server.",
                )
                .await?;
        }
    }
    Ok(())
}

#[command]
#[max_args(1)]
#[min_args(1)]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_default();
    if (admins.contains(&msg.author.id) || msg.author.id == OWNER_ID) && !msg.mentions.is_empty() {
        if let Some(user) = msg
            .mentions
            .iter()
            .find(|&user| user.id != BOT_ID && user.id != OWNER_ID)
        {
            if admins.contains(&user.id) {
                remove_admin(ctx, msg.guild_id, user.id).await?;
            } else {
                msg.channel_id
                    .say(ctx, "This user is not a bot admin on this server!")
                    .await?;
            }
        } else {
            msg.channel_id
                .say(
                    ctx,
                    "Mention a user you wish to remove from bot admins for this server.",
                )
                .await?;
        }
    }
    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id)
        .await
        .unwrap_or_else(|| vec![OWNER_ID]);
    let mut user_names: Vec<String> = vec![];
    for user in admins.iter().map(|id| id.to_user(ctx)) {
        let user = user.await?;
        user_names.push(user.name);
    }
    user_names.push(OWNER_ID.to_user(ctx).await?.name);
    let guild_name = msg
        .guild_id
        .unwrap_or(LOTR_DISCORD)
        .to_partial_guild(ctx)
        .await?
        .name;
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("List of bot admins");
                e.description(format!("On **{}**\n{}", guild_name, user_names.join("\n")))
            });
            m
        })
        .await?;
    Ok(())
}

#[command]
async fn floppadd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.author.id == OWNER_ID {
        let url = args.single::<String>();
        if let Ok(floppa_url) = url {
            add_floppa(ctx, floppa_url).await?;
            msg.channel_id
                .say(ctx, "Successfully added floppa to the database")
                .await?;
        }
    }
    Ok(())
}
