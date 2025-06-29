use serde::{Deserialize, Serialize};
use serenity::client::Context;

use crate::constants::MINECRAFT_API;
use crate::get_reqwest_client;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Description {
    pub raw: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub name_clean: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlayerList {
    pub online: u32,
    pub max: u32,
    #[serde(default)]
    pub list: Vec<Player>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftServer {
    pub online: bool,
    #[serde(default)]
    pub motd: Description,
    #[serde(default)]
    pub players: PlayerList,
}

pub async fn get_server_status(ctx: &Context, ip: &str) -> Option<MinecraftServer> {
    let rclient = get_reqwest_client!(ctx);

    let request = format!("{MINECRAFT_API}{ip}");
    let response = rclient.get(&request).send().await.ok()?.text().await.ok()?;
    match serde_json::from_str::<MinecraftServer>(&response) {
        Ok(server) => {
            if server.online {
                Some(server)
            } else {
                None
            }
        }
        Err(e) => {
            println!("Error parsing server status for {request}:\n{e} ({e:?})");
            println!("(answered: {response:?})");
            None
        }
    }
}
