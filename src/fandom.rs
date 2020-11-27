use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{id::UserId, prelude::Message};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

const BOT_ID: UserId = UserId(780858391383638057);

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<reqwest::Client>;
}

#[derive(Serialize, Deserialize)]
struct SearchPage {
    pageid: u64,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct SearchQuery {
    search: Vec<SearchPage>,
}

#[derive(Serialize, Deserialize)]
struct SearchRes {
    query: SearchQuery,
}

#[derive(Serialize, Deserialize)]
struct RandomPage {
    id: u64,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct RandomQuery {
    random: Vec<RandomPage>,
}

#[derive(Serialize, Deserialize)]
struct RandomRes {
    query: RandomQuery,
}

#[derive(Serialize, Deserialize)]
struct ImageServing {
    imageserving: String,
}

#[derive(Serialize, Deserialize)]
struct ImageRes {
    image: ImageServing,
}

pub struct GenericPage {
    pub id: u64,
    pub title: String,
}

impl From<SearchPage> for GenericPage {
    fn from(page: SearchPage) -> GenericPage {
        GenericPage {
            id: page.pageid,
            title: page.title,
        }
    }
}

impl From<RandomPage> for GenericPage {
    fn from(page: RandomPage) -> GenericPage {
        GenericPage {
            id: page.id,
            title: page.title,
        }
    }
}

pub enum Wikis {
    LOTRMod,
    // TolkienGateway,
}

impl Wikis {
    fn get_api(&self) -> &str {
        match self {
            Wikis::LOTRMod => "https://lotrminecraftmod.fandom.com/api.php?",
            /* Wikis::TolkienGateway => "http://tolkiengateway.net/w/api.php?", */ // Their api is too messy
        }
    }
}

pub enum Namespace {
    Page,
    User,
    File,
    Template,
    Category,
}

impl From<&Namespace> for u32 {
    fn from(namespace: &Namespace) -> u32 {
        match namespace {
            Namespace::Page => 0,
            Namespace::User => 2,
            Namespace::File => 6,
            Namespace::Template => 10,
            Namespace::Category => 14,
        }
    }
}
impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Namespace::Page => write!(f, "page"),
            Namespace::User => write!(f, "user"),
            Namespace::File => write!(f, "file"),
            Namespace::Template => write!(f, "template"),
            Namespace::Category => write!(f, "category"),
        }
    }
}

pub async fn search(
    ctx: &Context,
    ns: &Namespace,
    srsearch: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let ns: u32 = ns.into();

    let req = [
        ("format", "json"),
        ("action", "query"),
        ("list", "search"),
        ("srwhat", "text"),
        ("srlimit", "3"),
        ("srsearch", srsearch),
        ("srnamespace", &ns.to_string()),
    ];

    let res = fclient
        .get(wiki.get_api())
        .query(&req)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let body: SearchRes = serde_json::from_str(res.as_str()).ok()?;
    Some(body.query.search.into_iter().next()?.into())
}

pub async fn random(ctx: &Context, wiki: &Wikis) -> Option<GenericPage> {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let req = [
        ("format", "json"),
        ("action", "query"),
        ("list", "random"),
        ("rnnamespace", "0"),
        ("rnlimit", "3"),
    ];

    let res = fclient
        .get(wiki.get_api())
        .query(&req)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let body: RandomRes = serde_json::from_str(res.as_str()).ok()?;
    Some(body.query.random.into_iter().next()?.into())
}

pub async fn display(
    ctx: &Context,
    msg: &Message,
    page: &GenericPage,
    wiki: &Wikis,
) -> CommandResult {
    let (id, title) = (page.id, page.title.as_str());

    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let req = [
        ("format", "json"),
        ("action", "imageserving"),
        ("wisId", &id.to_string()),
    ];

    let res = fclient
        .get(wiki.get_api())
        .query(&req)
        .send()
        .await?
        .text()
        .await?;

    let body: Result<ImageRes, _> = serde_json::from_str(res.as_str());
    let img = if let Ok(body) = body {
        body.image.imageserving
    } else {
        String::from("https://static.wikia.nocookie.net/lotrminecraftmod/images/8/8e/GrukRenewedLogo.png/revision/latest")
    };

    let bot_icon = BOT_ID.to_user(ctx).await?.face();

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.icon_url(bot_icon);
                    a.name("The Lord of the Rings Minecraft Mod Wiki");
                    a.url("https://lotrminecraftmod.fandom.com/");
                    a
                });
                e.title(title);
                e.image(img);
                e.url(format!(
                    "https://lotrminecraftmod.fandom.com/wiki/{}",
                    title.replace(" ", "_")
                ));
                e
            })
        })
        .await?;

    Ok(())
}
