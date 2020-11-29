pub mod structures;

use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{id::UserId, prelude::Message};

use structures::*;

use structures::{Lang::*, Namespace::*};

const BOT_ID: UserId = UserId(780858391383638057);

pub async fn search(
    ctx: &Context,
    ns: &Namespace,
    srsearch: &str,
    wiki: &Wikis,
) -> Option<GenericPage> {
    println!("srsearch {}", srsearch);
    println!("namespace {}", ns);
    if !(wiki.get_lang()? == &En) || ns == &Page {
        let [title, link, desc] = google_search(ctx, srsearch, &wiki).await?;
        println!(
            "page-title {}\npage-link {}\npage-desc {}",
            title, link, desc
        );

        let lang = link
            .split("//")
            .into_iter()
            .nth(1)?
            .split('/')
            .into_iter()
            .nth(1)?;

        println!("page-lang {}", lang);

        let query = {
            let mut hit_title = if title.contains('|') {
                title.split('|')
            } else {
                title.split('-')
            };
            match hit_title.next()? {
                "Fandom" => hit_title.next()?,
                other => other,
            }
            .trim()
        };

        println!("page-query {}", query);

        if wiki.get_lang()? == &En {
            let fclient = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<ReqwestClient>()
                    .expect("Expected DatabasePool in TypeMap")
                    .clone()
            };

            let ns_code: String = ns.into();

            println!("page-en-query {}.", query);

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

            println!("page-en-title {}.", page.title);

            if page.title == query {
                println!("returning");
                return Some(GenericPage {
                    title: page.title,
                    link,
                    desc: Some(desc),
                    id: Some(page.pageid),
                });
            } else {
                return None;
            }
        } else {
            return Some(GenericPage {
                title: query.into(),
                link,
                desc: Some(desc),
                id: None,
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

        println!("srsearch {}", srsearch);

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
            id: Some(page.pageid),
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
    println!("display");
    let fclient = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ReqwestClient>()
            .expect("Expected DatabasePool in TypeMap")
            .clone()
    };

    let img = {
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
    }
    .unwrap_or_else(|| {
        "https://static.wikia.nocookie.net/lotrminecraftmod/images/8/8e/GrukRenewedLogo.png".into()
    });

    let bot_icon = BOT_ID.to_user(ctx).await?.face();

    let lang = wiki.get_lang().unwrap_or(&En);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.icon_url(bot_icon);
                    a.name(lang.main());
                    a.url(wiki.site())
                });
                e.title(&page.title);
                if let Some(desc) = &page.desc {
                    e.description(desc);
                    println!("embed-desc {}", desc);
                };
                e.image(&img);
                println!("embed-image {}", &img);
                e.url(&page.link);
                println!("embed-link {}", &page.link);
                e
            });
            m
        })
        .await?;

    Ok(())
}

pub async fn google_search(ctx: &Context, query: &str, wiki: &Wikis) -> Option<[String; 3]> {
    println!("google-search {}", query);
    let fclient = {
        let data_read = ctx.data.read().await;
        search_with_google::Client {
            client: data_read.get::<ReqwestClient>()?.clone(),
        }
    };
    let results = fclient
        .search(&format!("site:{} {}", wiki.site(), query), 3, None)
        .await;

    if let Ok(hits) = results {
        let hit = hits.get(0)?;
        Some([hit.title.clone(), hit.link.clone(), hit.description.clone()])
    } else {
        None
    }
}
