//! Constants used in commands and API bindings

use serenity::model::prelude::*;

/// User ID of the bot
pub const BOT_ID: UserId = UserId(780858391383638057);
/// User ID of the owner
pub const OWNER_ID: UserId = UserId(405421991777009678);
/// Guild ID of the [LOTR Mod Community Discord](https://discord.gg/QXkZzKU)
pub const LOTR_DISCORD: GuildId = GuildId(405091134327619587);

/// Maximum size, in bytes, of a JSON file for [announcements][crate::announcement]
/// and [custom commands][crate::commands::custom_commands]
pub const MAX_JSON_FILE_SIZE: u64 = 10240;

/// Set of [permissions][Permissions] needed to manage the bot (bot admins and owner excepted).
///
/// Equivalent to `ADMINISTRATOR | MANAGE_GUILD | MANAGE_CHANNELS`
pub const MANAGE_BOT_PERMS: Permissions = Permissions {
    bits: Permissions::ADMINISTRATOR.bits
        | Permissions::MANAGE_GUILD.bits
        | Permissions::MANAGE_CHANNELS.bits,
};

/// [LOTR Mod Wiki](https://lotrminecraftmod.fandom.com) adress
pub const WIKI_DOMAIN: &str = "lotrminecraftmod.fandom.com";

/// A Curseforge [public API](https://www.cfwidget.com/) for the
/// [`!curseforge`][crate::commands::general::curseforge] command
pub const CURSE_API: &str = "https://api.cfwidget.com/minecraft/mc-mods/";
/// A Minecraft server [public API](https://api.mcsrvstat.us/) for the [`!online`][crate::commands::servers::online] command
pub const MINECRAFT_API: &str = "https://api.mcsrvstat.us/2/";
/// Google API for custom google search
pub const GOOGLE_API: &str = "https://www.googleapis.com/customsearch/v1?";

/// Curseforge project ID for the [LOTR Mod Renewed](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-renewed)
pub const CURSEFORGE_ID_RENEWED: u32 = 406893;
/// Curseforge project ID for the [LOTR Mod Legacy](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-legacy)
pub const CURSEFORGE_ID_LEGACY: u32 = 423748;

/// SQL table name for the bot [prefix][crate::database::config]
pub const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";
/// SQL table name for [bot admins][crate::database::admin_data]
pub const TABLE_ADMINS: &str = "bot_admins";
/// SQL table name for [floppa][crate::database::floppa] pictures
pub const TABLE_FLOPPA: &str = "floppa_images";
/// SQL table name for the user [blacklist][crate::database::blacklist]
pub const TABLE_USER_BLACKLIST: &str = "user_blacklist";
/// SQL table name for the channel [blacklist][crate::database::blacklist]
pub const TABLE_CHANNEL_BLACKLIST: &str = "channel_blacklist";
/// SQL table name for Minecraft [servers IPs][crate::database::config]
pub const TABLE_MC_SERVER_IP: &str = "mc_server_ip";
/// SQL table name for [custom commands][crate::database::custom_commands]
pub const TABLE_CUSTOM_COMMANDS: &str = "custom_commands";
/// SQL table name for [bug reports][crate::database::bug_reports]
pub const TABLE_BUG_REPORTS: &str = "bug_reports";
/// SQL table name for [bug report links][crate::database::bug_reports]
pub const TABLE_BUG_REPORTS_LINKS: &str = "bug_reports__links";
/// SQL table name for [role handling][crate::database::roles]
pub const TABLE_ROLES: &str = "roles";
/// SQL table name for [role aliases handling][crate::database::roles]
pub const TABLE_ROLES_ALIASES: &str = "roles__aliases";
/// SQL table name for guild list and database cleanup
pub const TABLE_LIST_GUILDS: &str = "list_guilds";

/// Reserved command names that cannot be used as [custom commands][crate::commands::custom_commands]
pub const RESERVED_NAMES: [&str; 45] = [
    "legacy",
    "renewed",
    "download",
    "curseforge",
    "forge",
    "coremod",
    "invite",
    "discord",
    "facebook",
    "fb",
    "donate",
    "donation",
    "paypal",
    "prefix",
    "admin",
    "admins",
    "blacklist",
    "announce",
    "floppadmin",
    "guilds",
    "listguilds",
    "command",
    "custom_command",
    "define",
    "help",
    "floppa",
    "aeugh",
    "dagohon",
    "floppadd",
    "online",
    "ip",
    "server_ip",
    "wiki",
    "tolkien",
    "tolkiengateway",
    "mc",
    "minecraft",
    "track",
    "bug",
    "bugs",
    "buglist",
    "resolve",
    "clean_database",
    "user",
    "user_info",
];
