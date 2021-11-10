use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::{CommandError, CommandResult};

use crate::constants::CURSE_API;
use crate::get_reqwest_client;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseImage {
    pub is_default: bool,
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
pub struct CurseProject {
    pub id: u64,
    pub name: String,
    pub summary: String,
    pub website_url: String,
    #[serde(default)]
    pub attachments: Vec<CurseImage>,
    #[serde(default)]
    pub latest_files: Vec<CurseFile>,
    pub download_count: f64,
}

pub async fn get_project_info(ctx: &Context, id: u64) -> CommandResult<CurseProject> {
    let rclient = get_reqwest_client!(ctx);
    let req = format!("{}{}", CURSE_API, id);
    let res = rclient.get(&req).send().await?.text().await?;
    serde_json::from_str(&res).map_err(CommandError::from)
}
