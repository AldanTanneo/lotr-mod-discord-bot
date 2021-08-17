use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::utils::Colour;

use super::google::google_search;
use super::{GenericPage, Namespace, Namespace::*, RandomRes, Wikis, Wikis::*};
use crate::get_reqwest_client;

pub async fn search(
    ctx: &Context,
    namespace: &Namespace,
    query: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    let rclient = get_reqwest_client!(ctx);

    println!("wiki search: \"{}\" on {:?} ({})", query, wiki, namespace);

    if namespace == &Page {
        let [hit, link, desc] = google_search(ctx, query, wiki).await?;
        let query = hit
            .split(" | ")
            .flat_map(|sub| sub.split(" - "))
            .flat_map(|sub| sub.split(" – "))
            .find(|&part| !part.contains("Fandom"))?
            .trim();

        let ns_code: &str = namespace.into();

        let req = [
            ("format", "json"),
            ("action", "opensearch"),
            ("limit", "3"),
            ("redirects", "resolve"),
            ("search", query),
            ("namespace", ns_code),
        ];

        let res = rclient
            .get(wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let res: Value = serde_json::from_str(&res).ok()?;
        let title = res[1][0].as_str()?;

        if title == query {
            println!("result: \"{}\"", title);
            Some(GenericPage {
                title: title.into(),
                link,
                desc: Some(desc.replace(" \n", " ").replace("\n", " ")),
            })
        } else {
            None
        }
    } else {
        let ns_code: &str = namespace.into();

        let req = [
            ("format", "json"),
            ("action", "opensearch"),
            ("limit", "3"),
            ("redirects", "resolve"),
            ("search", query),
            ("namespace", ns_code),
        ];

        let res = rclient
            .get(wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let res: Value = serde_json::from_str(&res).ok()?;
        let title = res[1][0].as_str()?;

        println!("result: \"{}\"", title);

        Some(GenericPage {
            title: title.into(),
            link: format!("{}/{}", wiki.site(), title.replace(" ", "_")),
            desc: None,
        })
    }
}

pub async fn random(ctx: &Context, wiki: &Wikis) -> Option<GenericPage> {
    let rclient = get_reqwest_client!(ctx);

    let req = [
        ("format", "json"),
        ("action", "query"),
        ("list", "random"),
        ("rnnamespace", "0"),
        ("rnlimit", "3"),
    ];

    let res = rclient
        .get(wiki.get_api())
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
    let rclient = get_reqwest_client!(ctx, Result);

    let img = match wiki {
        LotrMod(_) | Minecraft => {
            let req = [
                ("format", "json"),
                ("action", "imageserving"),
                ("wisTitle", &page.title),
            ];

            let res = rclient
                .get(wiki.get_api())
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
                .get(wiki.get_api())
                .query(&req)
                .send()
                .await?
                .text()
                .await?;

            let body = serde_json::from_str::<Value>(&res).unwrap_or_default();

            let id = body["query"]["pageids"][0].as_str().unwrap_or("0");
            let pages = &body["query"]["pages"];

            pages[id]["imageinfo"][0]["url"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| wiki.default_img())
        }
    };

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
