//! Constants used in commands and API bindings
use serenity::model::prelude::*;

use crate::serenity;

/// User ID of the bot
pub const BOT_ID: UserId = UserId(780858391383638057);
/// User ID of the owner
pub const OWNER_ID: UserId = UserId(405421991777009678);
/// Guild ID of the [LOTR Mod Community Discord](https://discord.gg/QXkZzKU)
pub const LOTR_DISCORD: GuildId = GuildId(405091134327619587);

/// Set of [permissions][Permissions] needed to manage the bot (bot admins and owner excepted).
///
/// Equivalent to `ADMINISTRATOR | MANAGE_GUILD | MANAGE_CHANNELS`
pub const MANAGE_BOT_PERMS: Permissions = Permissions {
    bits: Permissions::ADMINISTRATOR.bits
        | Permissions::MANAGE_GUILD.bits
        | Permissions::MANAGE_CHANNELS.bits,
};

pub mod discord_colours {
    use poise::serenity::utils::Colour;

    pub const BLURPLE: Colour = Colour(0x5865F2);
    pub const GREEN: Colour = Colour(0x57F287);
    pub const YELLOW: Colour = Colour(0xFEE75C);
    pub const FUCHSIA: Colour = Colour(0xEB459E);
    pub const RED: Colour = Colour(0xED4245);
    pub const WHITE: Colour = Colour(0xFFFFFF);
    pub const BLACK: Colour = Colour(0x000000);
}
