use serde::{Deserialize, Serialize};
use serenity::client::Context;
use std::env;

use super::wiki::structures::Wikis;
use crate::constants::GOOGLE_API;
use crate::get_reqwest_client;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleSearch {
    pub items: Vec<SearchResult>,
}

pub async fn google_search(ctx: &Context, query: &str, wiki: &Wikis) -> Option<[String; 3]> {
    let api_key = env::var("GOOGLE_API_KEY").expect("Expected a google api key in the environment");
    let search_engine_id =
        env::var("GOOGLE_CX").expect("Expected a google search engine id in the environment");
    let rclient = get_reqwest_client!(ctx);

    let req: [(&str, &str); 5] = [
        ("key", &api_key),
        ("cx", &search_engine_id),
        ("q", &query.replace(' ', "+")),
        ("num", "3"),
        ("siteSearch", &wiki.site()),
    ];

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
    let hit = result.items.first()?;
    println!("Google search result: {}", hit.link);
    Some([hit.title.clone(), hit.link.clone(), hit.snippet.clone()])
}
