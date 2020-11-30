pub mod structures;

use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;

use structures::*;

use structures::{Namespace::*, Wikis::*};

pub async fn search(
    ctx: &Context,
    ns: &Namespace,
    srsearch: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    let lang = wiki.get_lang();
    if ns == &Page {
        let [hit, link, desc] = google_search(ctx, srsearch, &wiki).await?;
        println!("hit '{}'", hit);
        let query = hit
            .split(" | ")
            .flat_map(|sub| sub.split(" - "))
            .find(|part| !part.contains("Fandom"))?
            .trim();

        println!("query '{}'", query);

        if lang.is_some() {
            let fclient = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<ReqwestClient>()
                    .expect("Expected DatabasePool in TypeMap")
                    .clone()
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

            let res = fclient
                .get(&wiki.get_api())
                .query(&req)
                .send()
                .await
                .ok()?
                .text()
                .await
                .ok()?;

            let body: Value = serde_json::from_str(&res).ok()?;
            let title = body.get(1)?.get(0)?.as_str()?.trim();

            println!("title '{}'", title);

            if title == query {
                Some(GenericPage {
                    title: title.into(),
                    link,
                    desc: Some(desc),
                })
            } else {
                None
            }
        } else {
            Some(GenericPage {
                title: query.into(),
                link,
                desc: Some(desc),
            })
        }
    } else {
        let fclient = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<ReqwestClient>()
                .expect("Expected DatabasePool in TypeMap")
                .clone()
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

        let res = fclient
            .get(&wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let body: Value = serde_json::from_str(&res).ok()?;
        let title = body.get(1)?.get(0)?.as_str()?;

        Some(GenericPage {
            link: format!("{}/{}", wiki.site(), title.replace(" ", "_")),
            title: title.into(),
            desc: None,
        })
    }
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
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
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

            let res = fclient
                .get(&wiki.get_api())
                .query(&req)
                .send()
                .await?
                .text()
                .await?;

            let body: Result<ImageRes, _> = serde_json::from_str(&res);
            if let Ok(body) = body {
                Some(body.image.imageserving)
            } else {
                None
            }
            .unwrap_or_else(|| wiki.default_img())
        }
        TolkienGateway => wiki.default_img(),
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
            m
        })
        .await?;

    Ok(())
}

pub async fn google_search(ctx: &Context, query: &str, wiki: &Wikis) -> Option<[String; 3]> {
    let fclient = {
        let data_read = ctx.data.read().await;
        search_with_google::Client {
            client: data_read.get::<ReqwestClient>()?.clone(),
        }
    };

    let req = format!("site:{} {}", wiki.site(), query);
    println!("google {}", req);
    let results = fclient.search(&req, 2, None).await;

    if let Ok(hits) = results {
        let hit = hits.get(0)?;
        Some([hit.title.clone(), hit.link.clone(), hit.description.clone()])
    } else {
        None
    }
}
