use itertools::free::join;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::{
    channel::Message,
    gateway::{Activity, Ready},
    id::UserId,
};

use std::env;

const BOT_ID: u64 = 780858391383638057;

#[group]
#[commands(renewed, help, wiki, prefix)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let game =
            Activity::playing("The Lord of the Rings Mod: Bringing Middle-earth to Minecraft");
        ctx.set_activity(game).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_user_id(UserId(BOT_ID)) {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.content(format!("My prefix here is \"{}\"", get_prefix()))
                })
                .await
                .expect("Failed to send message");
        }
    }
}

fn get_prefix() -> String {
    // reads the bot prefix from environment variables
    env::var("PREFIX")
        .expect("Expected a prefix in the environment")
        .to_string()
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(|_, _| Box::pin(async move { Some(get_prefix()) }))
                .allow_dm(false)
                .on_mention(Some(UserId(780858391383638057)))
        })
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

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
    msg.author
        .direct_message(ctx, |m| {
            m.content(format!("My prefix here is \"{}\"", get_prefix()));
            m.embed(|e| {
                e.title("Available commands");
                e.description(
                    "renewed
wiki
help
prefix",
                );
                e
            });
            m
        })
        .await?;

    Ok(())
}

#[command]
async fn wiki(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = join(
        args.rest().split_whitespace().map(|word| {
            let (a, b) = word.split_at(1);
            format!("{}{}", a.to_uppercase(), b)
        }),
        "_",
    );

    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!("https://lotrminecraftmod.fandom.com/{}", url))
            /* m.embed(|e| {
                e.title(if url.is_empty() {
                    String::from("The Lord of the Rings Minecraft Mod Wiki")
                } else {
                    url.replace("_", " ")
                });
                e.url(format!("https://lotrminecraftmod.fandom.com/wiki/{}", url));
                e
            }) */
        })
        .await?;
    msg.delete(ctx).await?;

    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
#[max_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .send_message(ctx, |m| {
                m.content(format!("My prefix here is \"{}\"", get_prefix()))
            })
            .await?;
    } else {
        let new_prefix = args.single::<String>();
        if let Ok(p) = new_prefix {
            env::set_var("PREFIX", &p);
            std::process::Command::new("heroku")
                .arg("config:set")
                .arg(format!("PREFIX={}", &p));
            msg.channel_id
                .send_message(ctx, |m| {
                    m.content(format!("Set the new prefix to {}", get_prefix()))
                })
                .await?;
        } else {
            msg.channel_id
                .send_message(ctx, |m| m.content("Failed to set the new prefix!"))
                .await?;
        }
    }
    Ok(())
}
