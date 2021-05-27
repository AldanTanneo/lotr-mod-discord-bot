use async_minecraft_ping::{ConnectionConfig, ServerDescription::*};
use lazy_static::lazy_static;
use regex::Regex;

use super::MinecraftServer;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"(§[[:xdigit:]klmnor])?(§[[:xdigit:]klmnor])|([^§[:xdigit:]klmnor][^§]*)+")
            .unwrap();
}

fn parse_motd<T: Into<String>>(motd: T) -> String {
    let motd = motd.into();
    let mut res = String::with_capacity(motd.len());
    let matches = RE.find_iter(&motd);
    let mut stack: Vec<&str> = Vec::new();
    for m in matches.map(|m| m.as_str()) {
        match m {
            "§0" | "§1" | "§2" | "§3" | "§4" | "§5" | "§6" | "§7" | "§8" | "§9" | "§a" | "§b"
            | "§c" | "§d" | "§e" | "§f" | "§r" | "§k" => {
                stack.drain(..).rev().for_each(|t| res.push_str(t));
            }
            "§l" => {
                stack.push("**");
                res.push_str("**");
            }
            "§m" => {
                stack.push("~~");
                res.push_str("~~");
            }
            "§n" => {
                stack.push("__");
                res.push_str("__");
            }
            "§o" => {
                stack.push("*");
                res.push('*');
            }
            _ => res.push_str(m),
        }
    }
    stack.drain(..).rev().for_each(|t| res.push_str(t));
    res
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_motd_parser() {
        assert_eq!(
            super::parse_motd("§6The §nLord of §othe Rings§r"),
            "The __Lord of *the Rings*__"
        );
    }
}

pub async fn get_server_status(ip: &str) -> Option<MinecraftServer> {
    let (host, port): (&str, u16);
    if let Some((h, p)) = ip.rsplit_once(':') {
        host = h;
        port = p.parse().ok()?
    } else {
        host = ip;
        port = 25565;
    }

    let config = ConnectionConfig::build(host).with_port(port);

    match config.connect().await {
        Ok(mut conn) => match conn.status().await {
            Ok(status) => {
                let motd = parse_motd(match &status.description {
                    Plain(motd) => motd,
                    Object { text } => text,
                });
                let player_sample = status
                    .players
                    .sample
                    .map(|v| v.iter().map(|p| p.name.clone()).collect::<Vec<String>>());
                Some(MinecraftServer {
                    motd,
                    player_sample,
                    online_players: status.players.online,
                    max_players: status.players.max,
                })
            }
            Err(e) => {
                println!("Error fetching Minecraft server status: {}", e);
                None
            }
        },
        Err(e) => {
            println!("Error fetching Minecraft server status: {}", e);
            None
        }
    }
}
