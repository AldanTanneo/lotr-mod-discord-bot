use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Context, Result};

pub const CURSE_API: &str = "https://api.curseforge.com/v1/mods/";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseModLinks {
    pub website_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseModAsset {
    pub thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseFile {
    pub id: u64,
    pub file_name: String,
    pub file_length: u64,
    pub file_date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseMod {
    pub id: u64,
    pub name: String,
    pub summary: String,
    pub links: CurseModLinks,
    pub logo: CurseModAsset,
    #[serde(default)]
    pub latest_files: Vec<CurseFile>,
    pub download_count: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseProject {
    pub data: CurseMod,
}

pub async fn get_project_info(ctx: &Context<'_>, id: u64) -> Result<CurseMod> {
    let rclient = ctx.data().reqwest_client();

    let req = format!("{}{}", CURSE_API, id);

    let res = rclient
        .get(&req)
        .header("accept", "application/json")
        .header("x-api-key", ctx.data().curseforge_api_key())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(serde_json::from_str::<CurseProject>(&res)?.data)
}
