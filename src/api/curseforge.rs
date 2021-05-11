use serenity::client::Context;

use super::CurseProject;
use crate::constants::CURSE_API;
use crate::get_reqwest_client;

pub async fn get_project_info(ctx: &Context, id: u32) -> Option<CurseProject> {
    let rclient = get_reqwest_client!(ctx);
    let req = format!("{}{}", CURSE_API, id);
    let res = rclient.get(&req).send().await.ok()?.text().await.ok()?;
    dbg!(serde_json::from_str(&res).ok())
}
