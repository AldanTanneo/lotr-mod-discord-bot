use async_minecraft_ping::{ConnectionConfig, ServerDescription::*};

use super::MinecraftServer;

fn parse_motd<T: ToString>(motd: T) -> String {
    let motd = motd.to_string();
    let mut res = String::with_capacity(motd.len());
    let mut stack: Vec<&str> = Vec::new();
    let mut is_token = false;
    for c in motd.chars() {
        if c == '§' {
            is_token = true;
        } else if is_token {
            is_token = false;
            match c {
                '0'..='9' | 'a'..='f' | 'k' | 'r' => {
                    stack.drain(..).rev().for_each(|s| res.push_str(s));
                }
                'l' => {
                    stack.push("**");
                    res.push_str("**");
                }

                'n' => {
                    stack.push("__");
                    res.push_str("__");
                }
                'm' => {
                    stack.push("~~");
                    res.push_str("~~");
                }
                'o' => {
                    stack.push("*");
                    res.push('*');
                }
                _ => {
                    res.push('§');
                    res.push(c)
                }
            }
        } else {
            res.push(c);
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
            super::parse_motd(
                "§aHypixel Network  §c[1.8-1.16]\n§e§lSKYBLOCK§c, §b§lBEDWARS §c§l+ §a§lMORE"
            ),
            "Hypixel Network  [1.8-1.16]\n**SKYBLOCK**, **BEDWARS ****+ ****MORE**"
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
