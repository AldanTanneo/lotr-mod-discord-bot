use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::{CommandError, CommandResult};
use std::env;

use crate::constants::CURSE_API;
use crate::get_reqwest_client;

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

pub async fn get_project_info(ctx: &Context, id: u64) -> CommandResult<CurseProject> {
    let api_key =
        env::var("CURSEFORGE_API_KEY").expect("Expected a curseforge api key in the environment");

    let rclient = get_reqwest_client!(ctx);
    let request = format!("{CURSE_API}{id}");
    let response = rclient
        .get(&request)
        .header("accept", "application/json")
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    serde_json::from_str(&response).map_err(CommandError::from)
}
