mod database;
mod fandom;

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
    misc::Mentionable,
    prelude::ReactionType,
};
use std::{env, sync::Arc};

use database::*;
use fandom::*;
use structures::*;
use structures::{Lang::*, Namespace::*};
use Blacklist::*;

const BOT_ID: UserId = UserId(780858391383638057);
const OWNER_ID: UserId = UserId(405421991777009678);
const LOTR_DISCORD: GuildId = GuildId(405091134327619587);
const WIKI_DOMAIN: &str = "lotrminecraftmod.fandom.com";

#[group]
#[commands(help, renewed, tos, curseforge, prefix, forge, coremod)]
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
#[only_in(guilds)]
#[commands(floppadd, blacklist)]
struct Moderation;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let game =
            Activity::playing("The Lord of the Rings Mod: Bringing Middle-earth to Minecraft");
        ctx.set_activity(game).await;
        let _ = OWNER_ID
            .to_user(&ctx)
            .await
            .unwrap()
            .direct_message(&ctx, |m| m.content("Bot started and ready!"))
            .await;
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
                    Box::pin(async move {
                        Some(
                            get_prefix(ctx, msg.guild_id)
                                .await
                                .unwrap_or_else(|| "!".into()),
                        )
                    })
                })
                .allow_dm(false)
                .on_mention(Some(BOT_ID))
                .owners(vec![OWNER_ID].into_iter().collect())
        })
        .group(&GENERAL_GROUP)
        .group(&MEME_GROUP)
        .group(&WIKI_GROUP)
        .group(&ADMIN_GROUP)
        .group(&MODERATION_GROUP);

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

// ------------------ GENERAL COMMANDS --------------------

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
            m.content(format!("My prefix here is \"{}\"", prefix.unwrap_or_else(|| "!".into())));
            m.embed(|e| {
                e.title("Available commands");
                e.field(
                    "General commands",
                    "`renewed`,\n`forge`,\n`coremod`,\n`tos`,\n`curseforge`,\n`help`\n",
                    true,
                );
                e.field(
                    "Wiki commands",
                    "`wiki`\n`wiki user`\n`wiki category`\n`wiki template`\n`wiki random`\n`wiki file`\n`wiki tolkien`\n`wiki minecraft`\n",
                    true
                );
                e.field(
                    "Admin commands",
                    "`prefix`\n`admin add`\n`admin remove`\n`admin list`\n`blacklist`\n",
                    true,
                );
                e.field(
                    "Syntax: `wiki [subcommand] [language] [search terms]`",
                    "Available languages: `en` (default), `de`, `fr`, `es`, `nl`, `ja`, `zh`, `ru`\n",
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
async fn forge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (version, mc) = if args.single::<String>().unwrap_or_default() == "legacy" {
        ("1558", "1.7.10")
    } else {
        ("31.2.31", "1.15.2")
    };
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Have you checked your Forge version?");
            e.description(format!("To function properly, the mod needs to run with Forge {} or later for Minecraft {}", version, mc));
            e.author(|a| {
                a.name("Minecraft Forge");
                a.icon_url("https://pbs.twimg.com/profile_images/778706890914095109/fhMDH9o6_400x400.jpg");
                a.url("http://files.minecraftforge.net/")
            })
        })
    })
    .await?;

    Ok(())
}

#[command]
async fn coremod(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| m.embed(|e| {
            e.title("Check your mod file extension!");
            e.description("Sometimes when downloading the mod with a browser like Firefox, the mod file is saved with a `.zip` extension instead of a `.jar`
When this happens, the mod will not function properly: among other things that will happen, mod fences and fence gates will not connect, and horses will go very slowly.

To fix this, go to your `/.minecraft/mods` folder and change the file extension!")
        }))
        .await?;
    Ok(())
}

// ------------------ MEME COMMANDS -----------------

macro_rules! check_allowed {
    ($ctx:expr, $msg:expr) => {
        let admins = get_admins($ctx, $msg.guild_id).await.unwrap_or_default();
        if !(admins.contains(&$msg.author.id)
            || $msg.author.id == OWNER_ID
            || !check_blacklist($ctx, $msg, false)
                .await
                .unwrap_or_else(|| IsBlacklisted(true))
                .is_blacklisted())
        {
            $msg.delete($ctx).await?;
            $msg.author
                .dm($ctx, |m| {
                    m.content("You are not allowed to use this command here.")
                })
                .await?;
            return Ok(());
        }
    };
}

#[command]
#[max_args(1)]
async fn floppa(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    check_allowed!(ctx, msg);
    let n = args.single::<u32>().ok();
    let url = if let Some(url) = get_floppa(ctx, n).await {
        url
    } else {
        "https://i.kym-cdn.com/photos/images/original/001/878/839/c6f.jpeg".to_string()
    };
    msg.channel_id.say(ctx, url).await?;
    Ok(())
}

#[command]
async fn aeugh(ctx: &Context, msg: &Message) -> CommandResult {
    check_allowed!(ctx, msg);
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file("https://cdn.discordapp.com/attachments/405122337139064834/782087543046668319/aeugh.mp4")
        })
        .await?;
    Ok(())
}

#[command]
async fn dagohon(ctx: &Context, msg: &Message) -> CommandResult {
    check_allowed!(ctx, msg);
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file("https://cdn.discordapp.com/attachments/405097997970702337/782656209987043358/dagohon.mp4")
        })
        .await?;
    Ok(())
}

// --------------------- WIKI COMMANDS -------------------------

async fn wiki_search(
    ctx: &Context,
    msg: &Message,
    args: Args,
    namespace: Namespace,
    wiki: &Wikis,
) -> CommandResult {
    let srsearch = args.rest();
    println!("srsearch {}", srsearch);
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

fn lang(mut args: Args) -> (Lang, Args, bool) {
    let mut default = false;
    let lang = match args
        .single::<String>()
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "en" | "english" => En,
        "fr" | "french" => Fr,
        "es" | "spanish" => Es,
        "de" | "german" => De,
        "nl" | "dutch" => Nl,
        "zh" | "chinese" => Zh,
        "ru" | "russian" => Ru,
        "ja" | "japanese" => Ja,
        a => {
            println!("{}", a);
            default = true;
            En
        }
    };
    (lang, args, default)
}

async fn lotr_wiki(ctx: &Context, msg: &Message, args: Args, ns: Namespace) -> CommandResult {
    let res = lang(args);
    let lang = res.0;
    let mut args = res.1;
    let default = res.2;
    let wiki = Wikis::LOTRMod(lang);
    if default {
        println!("rewinding");
        args.rewind();
    }
    if !args.is_empty() {
        wiki_search(ctx, msg, args, ns, &wiki).await?;
    } else {
        fandom::display(ctx, msg, &ns.main_page(&wiki, &msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
async fn wiki(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Page).await?;
    Ok(())
}

#[command]
async fn user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, User).await?;
    Ok(())
}

#[command]
async fn category(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Category).await?;
    Ok(())
}
#[command]
async fn template(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, Template).await?;
    Ok(())
}

#[command]
async fn file(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, args, File).await?;
    Ok(())
}

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::LOTRMod(En);
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
    let wiki = Wikis::TolkienGateway;
    if !args.is_empty() {
        wiki_search(ctx, msg, args, Page, &wiki).await?;
    } else {
        fandom::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
async fn minecraft(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let wiki = Wikis::Minecraft;
    if !args.is_empty() {
        wiki_search(ctx, msg, args, Page, &wiki).await?;
    } else {
        fandom::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
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
            .say(
                ctx,
                format!(
                    "My prefix here is \"{}\"",
                    prefix.unwrap_or_else(|| "!".into())
                ),
            )
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
                msg.react(ctx, ReactionType::from('✅')).await?;
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
    } else {
        msg.channel_id
            .say(ctx, "You are not an admin on this server!")
            .await?;
        msg.react(ctx, ReactionType::from('❌')).await?;
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
                msg.react(ctx, ReactionType::from('✅')).await?;
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
    } else {
        msg.channel_id
            .say(ctx, "You are not an admin on this server!")
            .await?;
        msg.react(ctx, ReactionType::from('❌')).await?;
    }
    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_else(Vec::new);

    let mut user_names: Vec<String> = admins.iter().map(|&id| id.mention()).collect();
    user_names.push(OWNER_ID.mention());

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
            msg.react(ctx, ReactionType::from('✅')).await?;
        }
    }
    Ok(())
}

#[command]
async fn blacklist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let admins = get_admins(ctx, msg.guild_id).await.unwrap_or_default();
    if admins.contains(&msg.author.id) || msg.author.id == OWNER_ID {
        println!("{:#?}, {:#?}", msg.mention_channels, msg.mentions);
        if args.is_empty() && msg.mentions.is_empty() {
            let (users, channels) = check_blacklist(ctx, msg, true)
                .await
                .unwrap_or(IsBlacklisted(true))
                .get_list();

            let mut user_names: Vec<String> = users.iter().map(|&u| u.mention()).collect();

            let mut channel_names: Vec<String> = channels.iter().map(|&c| c.mention()).collect();

            if user_names.is_empty() {
                user_names.push("None".into());
            }
            if channel_names.is_empty() {
                channel_names.push("None".into());
            }

            let guild_name = msg
                .guild_id
                .unwrap_or(LOTR_DISCORD)
                .to_partial_guild(ctx)
                .await?
                .name;
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.title("Blacklist");
                        e.description(format!("On **{}**", guild_name));
                        e.field("Blacklisted users:", user_names.join("\n"), true);
                        e.field("Blacklisted channels:", channel_names.join("\n"), true)
                    });
                    m
                })
                .await?;
        } else {
            update_blacklist(ctx, msg, args).await?;
        }
    } else {
        msg.channel_id
            .say(ctx, "You are not an admin on this server!")
            .await?;
        msg.react(ctx, ReactionType::from('❌')).await?;
    }
    Ok(())
}
