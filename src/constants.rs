//! Constants used in commands and API bindings
use crate::serenity;

use serenity::model::prelude::*;

/// User ID of the bot
pub const BOT_ID: UserId = UserId(780858391383638057);
/// User ID of the owner
pub const OWNER_ID: UserId = UserId(405421991777009678);
/// Guild ID of the [LOTR Mod Community Discord](https://discord.gg/QXkZzKU)
pub const LOTR_DISCORD: GuildId = GuildId(405091134327619587);

/// Set of [permissions][Permissions] needed to manage the bot (bot admins and owner excepted).
///
/// Equivalent to `ADMINISTRATOR | MANAGE_GUILD | MANAGE_CHANNELS`
pub const MANAGE_BOT_PERMS: Permissions = Permissions::ADMINISTRATOR
    .union(Permissions::MANAGE_GUILD)
    .union(Permissions::MANAGE_CHANNELS)
    .union(Permissions::MANAGE_ROLES)
    .union(Permissions::MANAGE_WEBHOOKS)
    .union(Permissions::MODERATE_MEMBERS);

/// Set of [permissions][Permissions] needed for the bot when inviting it to a server
pub const INVITE_PERMISSIONS: Permissions = Permissions::MANAGE_ROLES
    .union(Permissions::CHANGE_NICKNAME)
    .union(Permissions::READ_MESSAGES)
    .union(Permissions::SEND_MESSAGES)
    .union(Permissions::CREATE_PUBLIC_THREADS)
    .union(Permissions::CREATE_PRIVATE_THREADS)
    .union(Permissions::SEND_MESSAGES_IN_THREADS)
    .union(Permissions::MANAGE_MESSAGES)
    .union(Permissions::MANAGE_THREADS)
    .union(Permissions::EMBED_LINKS)
    .union(Permissions::ATTACH_FILES)
    .union(Permissions::READ_MESSAGE_HISTORY)
    .union(Permissions::MENTION_EVERYONE)
    .union(Permissions::USE_EXTERNAL_EMOJIS)
    .union(Permissions::USE_EXTERNAL_STICKERS)
    .union(Permissions::ADD_REACTIONS);

pub mod curseforge {
    use poise::serenity::utils::Colour;

    /// Curseforge logo
    pub const ICON: &str = "https://tinyimg.io/i/SVsK1qC.png";

    /// Curseforge orange accent colour
    pub const COLOUR: Colour = Colour(0xf16436);
}
