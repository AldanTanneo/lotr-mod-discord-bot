use json::{parse, JsonValue};
use reqwest;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<reqwest::Client>;
}

pub struct Page {
    pub id: u64,
    pub title: String,
}

const WIKI_API: &str = "https://lotrminecraftmod.fandom.com/api.php&format=json&";

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

pub async fn search(ctx: &Context, ns: &str, srsearch: &str) -> Option<Page> {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let req = [
        ("action", "query"),
        ("list", "search"),
        ("srwhat", "text"),
        ("srsearch", srsearch),
        ("srnamespace", namespace(ns)?),
        ("srlimit", "1"),
    ];

    let res = fclient
        .get(WIKI_API)
        .query(&req)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let mut body = parse(res.as_str()).ok()?;
    let mut results = match body["query"]["search"].take() {
        JsonValue::Array(v) => Some(v),
        _ => None,
    }?;
    if results.len() != 0 {
        match (
            results[0].take()["pageid"].take(),
            results[0].take()["title"].take(),
        ) {
            (JsonValue::Number(id), JsonValue::String(s)) => Some(Page {
                id: id.as_fixed_point_u64(0).unwrap(),
                title: s.to_string(),
            }),
            _ => None,
        }
    } else {
        None
    }
}

pub async fn random(ctx: &Context) -> Option<Page> {
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

    let body = parse(res.as_str()).ok()?;

    let (rnd_id, rnd_title) = (
        &body["query"]["random"]["id"],
        &body["query"]["random"]["title"],
    );

    match (rnd_id, rnd_title) {
        (JsonValue::Number(id), JsonValue::String(s)) => Some(Page {
            id: id.as_fixed_point_u64(0)?,
            title: s.to_string(),
        }),
        _ => None,
    }
}

pub async fn display(ctx: &Context, msg: &Message, p: Page) -> CommandResult {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };
    let res = fclient
        .get(WIKI_API)
        .query(&[("action", "imageserving"), ("wisId", &p.id.to_string())])
        .send()
        .await?
        .text()
        .await?;
    let body = parse(res.as_str())?;

    let img = &body["image"]["imageserving"];
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&p.title);
                e.image(img);
                e.url(format!(
                    "https://lotrminecraftmod.fandom.com/wiki/{}",
                    &p.title.replace(" ", "_")
                ));
                e
            })
        })
        .await?;

    Ok(())
}
