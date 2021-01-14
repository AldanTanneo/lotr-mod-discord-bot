use serenity::client::Context;

use super::structures::{CurseProject, ReqwestClient};
use crate::constants::CURSE_API;

pub async fn get_project_info(ctx: &Context, id: u32) -> Option<CurseProject> {
    let rclient = {
        let data_read = ctx.data.read().await;
        data_read.get::<ReqwestClient>()?.clone()
    };
    let req = format!("{}{}", CURSE_API, id);
    let res = rclient.get(&req).send().await.ok()?.text().await.ok()?;
    dbg!(serde_json::from_str(&res).ok())
}
