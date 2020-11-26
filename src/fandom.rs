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

    let res = fclient
        .get({
            let x = format!(
                "{}action=query&list=search&srwhat=text&srlimit=1&srsearch={}&srnamespace={}",
                WIKI_API,
                srsearch,
                namespace(ns)?
            )
            .as_str();
            println!("Search query: {}", x);
            x
        })
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    println!("Search: {}", res);

    let mut body = parse(res.as_str()).ok()?;

    println!("Parsed successfully");

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

    println!("Random: {}", res);

    let body = parse(res.as_str()).ok()?;

    println!("Parsed successfully");

    let (rnd_id, rnd_title) = (
        &body["query"]["random"][0]["id"],
        &body["query"]["random"][0]["title"],
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
        .await
        .expect("Oh no it broke at send")
        .text()
        .await
        .expect("Oh no it broke at text");

    println!("Display: {}", res);

    let mut body = parse(res.as_str())?;

    println!("Parsed successfully");

    let img = if let JsonValue::String(img) = body["image"].take()["imageserving"].take() {
        img
    } else {
        String::new()
    };
    println!("{}", img);
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
