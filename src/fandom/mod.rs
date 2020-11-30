pub mod structures;

use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;

use structures::*;

use structures::{Lang::*, Namespace::*, Wikis::*};

pub async fn search(
    ctx: &Context,
    ns: &Namespace,
    srsearch: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    let lang = wiki.get_lang();
    if !(lang == Some(&En)) || ns == &Page {
        let [title, link, desc] = google_search(ctx, srsearch, &wiki).await?;

        let query = {
            let mut hit_title = title.split(|c| ['|', '-', '–'].contains(&c));
            match hit_title.next()?.trim() {
                "Fandom" => hit_title.next()?,
                other => other,
            }
            .trim()
        };

        if lang == Some(&En) {
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
                ("action", "query"),
                ("list", "search"),
                ("srwhat", "nearmatch"),
                ("srlimit", "3"),
                ("srsearch", query),
                ("srnamespace", &ns_code),
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

            let body: SearchRes = serde_json::from_str(&res).ok()?;
            let page = body.query.search.into_iter().next()?;

            if page.title.contains(query) || query.contains(&page.title) {
                return Some(GenericPage {
                    title: page.title,
                    link,
                    desc: Some(desc),
                });
            } else {
                return None;
            }
        } else {
            return Some(GenericPage {
                title: query.into(),
                link,
                desc: Some(desc),
            });
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
            ("action", "query"),
            ("list", "search"),
            ("srwhat", "text"),
            ("srlimit", "3"),
            ("srsearch", srsearch),
            ("srnamespace", &ns_code),
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

        let body: SearchRes = serde_json::from_str(&res).ok()?;
        let page = body.query.search.into_iter().next()?;

        return Some(GenericPage {
            link: format!("{}/{}", wiki.site(), page.title.replace(" ", "_")),
            title: page.title,
            desc: None,
        });
    };
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
    Some(body.query.random.into_iter().next()?.into())
}

pub async fn display(
    ctx: &Context,
    msg: &Message,
    page: &GenericPage,
    wiki: &Wikis,
) -> CommandResult {
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let img = match wiki {
        LOTRMod(_) | Minecraft => {
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
    let results = fclient
        .search(&format!("site:{} {}", wiki.site(), query), 1, None)
        .await;

    if let Ok(hits) = results {
        let hit = hits.get(0)?;
        Some([hit.title.clone(), hit.link.clone(), hit.description.clone()])
    } else {
        None
    }
}
