use serenity::client::Context;

use super::structures::{MinecraftServer, ReqwestClient};
use crate::constants::MINECRAFT_API;

pub async fn get_server_status(ctx: &Context, ip: &String) -> Option<MinecraftServer> {
    let rclient = {
        let data_read = ctx.data.read().await;
        data_read.get::<ReqwestClient>()?.clone()
    };
    let req = format!("{}{}", MINECRAFT_API, ip);
    let res = rclient.get(&req).send().await.ok()?.text().await.ok()?;
    if let Ok(server) = serde_json::from_str::<MinecraftServer>(&res) {
        if server.online {
            Some(server)
        } else {
            None
        }
    } else {
        None
    }
}
