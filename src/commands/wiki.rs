use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::commands::general::*;
use crate::{api, failure};
use api::structures::{Lang, Lang::*, Namespace, Namespace::*, Wikis};
use api::wiki;

async fn wiki_search(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
    namespace: Namespace,
    wiki: &Wikis,
) -> CommandResult {
    let srsearch = args.rest();
    let p = wiki::search(ctx, &namespace, srsearch, wiki).await;
    if let Some(page) = p {
        wiki::display(ctx, msg, &page, wiki).await?;
    } else {
        failure!(
            ctx,
            msg,
            "Couldn't find a {} for the given name!",
            namespace
        );
    }
    Ok(())
}

fn lang(args: &mut Args) -> Option<Lang> {
    Some(
        match args.single::<String>().ok()?.to_lowercase().as_str() {
            "en" | "english" => En,
            "fr" | "french" => Fr,
            "es" | "spanish" => Es,
            "de" | "german" => De,
            "nl" | "dutch" => Nl,
            "zh" | "chinese" => Zh,
            "ru" | "russian" => Ru,
            "ja" | "japanese" => Ja,
            _ => {
                args.rewind();
                return None;
            }
        },
    )
}

async fn lotr_wiki(ctx: &Context, msg: &Message, args: &mut Args, ns: Namespace) -> CommandResult {
    let language = lang(args).unwrap_or_default();
    let wiki = Wikis::LotrMod(language);
    if !args.is_empty() {
        wiki_search(ctx, msg, args, ns, &wiki).await?;
    } else {
        wiki::display(ctx, msg, &ns.main_page(&wiki, &msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
#[sub_commands(discord, user, category, template, file, random, tolkien, minecraft)]
pub async fn wiki(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Page).await?;
    Ok(())
}

#[command]
async fn user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, User).await?;
    Ok(())
}

#[command]
async fn category(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Category).await?;
    Ok(())
}
#[command]
async fn template(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, Template).await?;
    Ok(())
}

#[command]
async fn file(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    lotr_wiki(ctx, msg, &mut args, File).await?;
    Ok(())
}

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let wiki = &Wikis::LotrMod(En);
    let p = wiki::random(ctx, wiki).await;
    if let Some(page) = p {
        wiki::display(ctx, msg, &page, wiki).await?;
    } else {
        failure!(ctx, msg, "Couldn't execute query!");
    }
    Ok(())
}

#[command]
#[aliases("tolkiengateway")]
pub async fn tolkien(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let wiki = Wikis::TolkienGateway;
    if !args.is_empty() {
        wiki_search(ctx, msg, &mut args, Page, &wiki).await?;
    } else {
        wiki::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}

#[command]
#[aliases("mc")]
pub async fn minecraft(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let wiki = Wikis::Minecraft;
    if !args.is_empty() {
        wiki_search(ctx, msg, &mut args, Page, &wiki).await?;
    } else {
        wiki::display(ctx, msg, &wiki.default(&msg.author.name), &wiki).await?;
    }
    Ok(())
}
