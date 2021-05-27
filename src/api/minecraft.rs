use serenity::client::Context;

use super::MinecraftServer;
use crate::constants::MINECRAFT_API;
use crate::get_reqwest_client;

pub async fn get_server_status(ctx: &Context, ip: &str) -> Option<MinecraftServer> {
    let rclient = get_reqwest_client!(ctx);

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
