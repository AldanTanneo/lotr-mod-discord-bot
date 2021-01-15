pub mod curseforge;
mod google;
pub mod minecraft;
pub mod structures;

use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::utils::Colour;

use google::google_search;
use structures::*;
use structures::{Namespace::*, Wikis::*};

pub async fn search(
    ctx: &Context,
    ns: &Namespace,
    srsearch: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    if ns == &Page {
        let [hit, link, desc] = google_search(ctx, srsearch, &wiki).await?;
        println!("hit '{}'", hit);
        let query = hit
            .split(" | ")
            .flat_map(|sub| sub.split(" - "))
            .flat_map(|sub| sub.split(" – "))
            .find(|part| !part.contains("Fandom"))?
            .trim();

        println!("query '{}'", query);
        let rclient = {
            let data_read = ctx.data.read().await;
            data_read.get::<ReqwestClient>()?.clone()
        };

        let ns_code: String = ns.into();

        let req = [
            ("format", "json"),
            ("action", "opensearch"),
            ("limit", "3"),
            ("redirects", "resolve"),
            ("search", query),
            ("namespace", &ns_code),
        ];

        let res = rclient
            .get(&wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let res: Value = serde_json::from_str(&res).ok()?;
        let title = res[1][0].as_str()?;

        println!("title '{}'", title);

        if title == query {
            Some(GenericPage {
                title: title.into(),
                link,
                desc: Some(desc.replace(" \n", "").replace("\n", " ")),
            })
        } else {
            None
        }
    } else {
        let rclient = {
            let data_read = ctx.data.read().await;
            data_read.get::<ReqwestClient>()?.clone()
        };

        let ns_code: String = ns.into();

        let req = [
            ("format", "json"),
            ("action", "opensearch"),
            ("limit", "3"),
            ("redirects", "resolve"),
            ("search", srsearch),
            ("namespace", &ns_code),
        ];

        let res = rclient
            .get(&wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let res: Value = serde_json::from_str(&res).ok()?;
        let title = res[1][0].as_str()?;

        Some(GenericPage {
            title: title.into(),
            link: format!("{}/{}", wiki.site(), title.replace(" ", "_")),
            desc: None,
        })
    }
}

pub async fn random(ctx: &Context, wiki: &Wikis) -> Option<GenericPage> {
    let rclient = {
        let data_read = ctx.data.read().await;
        data_read.get::<ReqwestClient>()?.clone()
    };

    let req = [
        ("format", "json"),
        ("action", "query"),
        ("list", "random"),
        ("rnnamespace", "0"),
        ("rnlimit", "3"),
    ];

    let res = rclient
        .get(&wiki.get_api())
        .query(&req)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let body: RandomRes = serde_json::from_str(&res).ok()?;
    Some(
        body.query
            .random
            .into_iter()
            .find(|p| !p.title.contains("/Recipes"))?
            .into(),
    )
}

pub async fn display(
    ctx: &Context,
    msg: &Message,
    page: &GenericPage,
    wiki: &Wikis,
) -> CommandResult {
    println!("display");
    let rclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected ReqwestClient in TypeMap")
            .clone()
    };

    let img = match wiki {
        LOTRMod(_) | Minecraft => {
            println!("imageserving");
            let req = [
                ("format", "json"),
                ("action", "imageserving"),
                ("wisTitle", &page.title),
            ];

            let res = rclient
                .get(&wiki.get_api())
                .query(&req)
                .send()
                .await?
                .text()
                .await?;

            let body = serde_json::from_str::<Value>(&res).unwrap_or_default();
            body["image"]["imageserving"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| wiki.default_img())
        }
        TolkienGateway => {
            println!("image from tolkiengateway");
            let req = [
                ("format", "json"),
                ("action", "query"),
                ("generator", "images"),
                ("gimlimit", "2"),
                ("titles", &page.title),
                ("prop", "imageinfo"),
                ("iiprop", "url"),
                ("indexpageids", "true"),
            ];

            let res = rclient
                .get(&wiki.get_api())
                .query(&req)
                .send()
                .await?
                .text()
                .await?;

            let body = serde_json::from_str::<Value>(&res).unwrap_or_default();

            let id = body["query"]["pageids"][0].as_str().unwrap_or("0");
            println!("id {:?}", id);
            let pages = &body["query"]["pages"];
            println!("body {:?}", pages);
            pages[id]["imageinfo"][0]["url"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| wiki.default_img())
        }
    };
    println!("img '{}'", img);
    println!(
        "page '{}'\n   '{}'\n   '{:?}'",
        page.title, page.link, page.desc
    );
    println!(
        "wiki '{}'\n   '{}'\n   '{}'",
        wiki.name(),
        wiki.site(),
        wiki.icon()
    );

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(Colour::BLUE);
                e.author(|a| {
                    a.icon_url(wiki.icon());
                    a.name(wiki.name());
                    a.url(wiki.site())
                });
                e.title(&page.title);
                if let Some(desc) = &page.desc {
                    e.description(desc);
                };
                e.image(&img);
                e.url(&page.link);
                e
            });
            m.reference_message(msg);
            m.allowed_mentions(|a| a.empty_parse())
        })
        .await?;

    Ok(())
}
