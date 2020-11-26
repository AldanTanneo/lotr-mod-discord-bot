use serde::{Deserialize, Serialize};
use serde_json;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<reqwest::Client>;
}

#[derive(Serialize, Deserialize)]
pub struct SearchPage {
    pub pageid: u64,
    pub title: String,
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
pub struct RandomPage {
    pub id: u64,
    pub title: String,
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
pub struct ImageServing {
    imageserving: String,
}

#[derive(Serialize, Deserialize)]
struct ImageRes {
    image: ImageServing,
}

const WIKI_API: &str = "https://lotrminecraftmod.fandom.com/api.php?format=json&";

pub fn namespace(s: &str) -> Option<&str> {
    match s {
        "Page" => Some("0"),
        "User" => Some("2"),
        "File" => Some("6"),
        "Template" => Some("10"),
        "Category" => Some("14"),
        _ => None,
    }
}

pub async fn search(ctx: &Context, ns: &str, srsearch: &str) -> Option<SearchPage> {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let req = format!(
        "{}action=query&list=search&srwhat=text&srlimit=1&srsearch={}&srnamespace={}",
        WIKI_API,
        srsearch,
        namespace(ns)?
    );
    println!("Search query: {}", &req);

    let res = fclient
        .get(req.as_str())
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    println!("Search: {}", res);

    let body: SearchRes = serde_json::from_str(res.as_str()).ok()?;

    println!("Parsed successfully");

    Some(body.query.search.into_iter().nth(0)?)
}

pub async fn random(ctx: &Context) -> Option<RandomPage> {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let res = fclient
        .get("https://lotrminecraftmod.fandom.com/api.php?action=query&list=random&rnnamespace=0&rnlimit=1&format=json")
        .send().await.ok()?.text().await.ok()?;

    println!("Random: {}", res);

    let body: RandomRes = serde_json::from_str(res.as_str()).ok()?;

    println!("Parsed successfully");

    Some(body.query.random.into_iter().nth(0)?)
}

pub async fn display(ctx: &Context, msg: &Message, id: u64, title: String) -> CommandResult {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let req = format!("{}action=imageserving&wisId={}", WIKI_API, id);

    println!("Display query: {}", &req);

    let res = fclient.get(req.as_str()).send().await?.text().await?;

    println!("Display: {}", res);

    let body: Result<ImageRes, _> = serde_json::from_str(res.as_str());

    println!("Parsed successfully");

    let img = if let Ok(body) = body {
        body.image.imageserving
    } else {
        String::from("https://static.wikia.nocookie.net/lotrminecraftmod/images/8/8e/GrukRenewedLogo.png/revision/latest?cb=20200914215540")
    };

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&title);
                e.image(img);
                e.url(format!(
                    "https://lotrminecraftmod.fandom.com/wiki/{}",
                    title.as_str().replace(" ", "_")
                ));
                e
            })
        })
        .await?;

    Ok(())
}
