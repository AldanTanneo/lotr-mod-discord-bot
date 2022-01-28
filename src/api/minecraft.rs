use serde::{Deserialize, Serialize};

use crate::{Context, Result};

pub const MINECRAFT_API: &str = "https://api.mcsrvstat.us/2/";

#[derive(Serialize, Deserialize, Debug)]
pub struct Description {
    pub raw: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerList {
    pub online: u32,
    pub max: u32,
    pub list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftServer {
    pub online: bool,
    pub motd: Option<Description>,
    pub players: PlayerList,
}

pub async fn get_server_status(ctx: &Context<'_>, ip: &str) -> Result<MinecraftServer> {
    let rclient = ctx.data().reqwest_client();

    let req = format!("{}{}", MINECRAFT_API, ip);
    let res = rclient
        .get(&req)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(serde_json::from_str::<MinecraftServer>(&res)?)
}
