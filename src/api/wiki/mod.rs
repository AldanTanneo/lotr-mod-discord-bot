pub mod structures;

use serde_json::Value;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::{application::component::ButtonStyle, channel::Message};
use serenity::utils::colours;

use crate::api::google::google_search;
use crate::get_reqwest_client;
use structures::{GenericPage, Namespace, Namespace::*, RandomRes, Wikis, Wikis::*};

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

        let request = [
            ("format", "json"),
            ("action", "opensearch"),
            ("limit", "3"),
            ("redirects", "resolve"),
            ("search", query),
            ("namespace", ns_code),
        ];

        let response = rclient
            .get(wiki.get_api())
            .query(&request)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let response = serde_json::from_str::<Value>(&response).ok()?;
        let title = response[1][0].as_str()?;

        if title == query {
            println!("result: \"{}\"", title);
            Some(GenericPage {
                title: title.into(),
                link,
                desc: Some(desc.replace(" \n", " ").replace('\n', " ")),
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

        let response = rclient
            .get(wiki.get_api())
            .query(&req)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let response = serde_json::from_str::<Value>(&response).ok()?;
        let title = response[1][0].as_str()?;

        println!("result: \"{}\"", title);

        Some(GenericPage {
            title: title.into(),
            link: format!("{}/{}", wiki.site(), title.replace(' ', "_")),
            desc: None,
        })
    }
}

pub async fn random(ctx: &Context, wiki: &Wikis) -> Option<GenericPage> {
    let rclient = get_reqwest_client!(ctx);

    let request = [
        ("format", "json"),
        ("action", "query"),
        ("list", "random"),
        ("rnnamespace", "0"),
        ("rnlimit", "3"),
    ];

    let response = rclient
        .get(wiki.get_api())
        .query(&request)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let body: RandomRes = serde_json::from_str(&response).ok()?;
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
    let rclient = get_reqwest_client!(ctx);

    let img = match wiki {
        LotrMod(_) | Minecraft => {
            let request = [
                ("format", "json"),
                ("action", "imageserving"),
                ("wisTitle", &page.title),
            ];

            let response = rclient
                .get(wiki.get_api())
                .query(&request)
                .send()
                .await?
                .text()
                .await?;

            let body = serde_json::from_str::<Value>(&response).unwrap_or_default();
            body["image"]["imageserving"]
                .as_str()
                .map_or_else(|| wiki.default_img(), String::from)
        }
        TolkienGateway => {
            let request = [
                ("format", "json"),
                ("action", "query"),
                ("generator", "images"),
                ("gimlimit", "2"),
                ("titles", &page.title),
                ("prop", "imageinfo"),
                ("iiprop", "url"),
                ("indexpageids", "true"),
            ];

            let response = rclient
                .get(wiki.get_api())
                .query(&request)
                .send()
                .await?
                .text()
                .await?;

            let body = serde_json::from_str::<Value>(&response).unwrap_or_default();

            let id = body["query"]["pageids"][0].as_str().unwrap_or("0");
            let pages = &body["query"]["pages"];

            pages[id]["imageinfo"][0]["url"]
                .as_str()
                .map_or_else(|| wiki.default_img(), String::from)
        }
    };

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(colours::branding::BLURPLE);
                e.author(|a| a.icon_url(wiki.icon()).name(wiki.name()).url(wiki.site()));
                e.title(&page.title);
                if let Some(desc) = &page.desc {
                    e.description(desc);
                };
                e.image(&img);
                e.url(&page.link);
                e
            })
            .reference_message(msg)
            .allowed_mentions(|a| a.empty_parse())
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.style(ButtonStyle::Link).label("See page").url(&page.link)
                    })
                })
            })
        })
        .await?;

    Ok(())
}
