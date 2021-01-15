use serenity::client::Context;
use std::env;

use super::structures::{GoogleSearch, ReqwestClient, Wikis};
use crate::constants::{GOOGLE_API, GOOGLE_CX};

pub async fn google_search(ctx: &Context, query: &str, wiki: &Wikis) -> Option<[String; 3]> {
    let api_key = env::var("GOOGLE_API_KEY").expect("Expected a google api key in the environment");
    let rclient = {
        let data_read = ctx.data.read().await;
        data_read.get::<ReqwestClient>()?.clone()
    };

    let req = [
        ("cx", GOOGLE_CX),
        ("key", &api_key),
        ("q", &query.replace(" ", "+")),
        ("num", "3"),
        ("siteSearch", &wiki.site()),
    ];
    println!("google {:?}", req);

    let res_body = rclient
        .get(GOOGLE_API)
        .query(&req)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let result: GoogleSearch = serde_json::from_str(&res_body).ok()?;
    let hit = result.items.get(0)?;
    Some([hit.title.clone(), hit.link.clone(), hit.snippet.clone()])
}
