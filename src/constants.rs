use serenity::model::id::{GuildId, UserId};
use serenity::model::Permissions;

pub const BOT_ID: UserId = UserId(780858391383638057);
pub const OWNER_ID: UserId = UserId(405421991777009678);
pub const LOTR_DISCORD: GuildId = GuildId(405091134327619587);

pub const MAX_JSON_FILE_SIZE: u64 = 25600;

pub const MANAGE_BOT_PERMS: Permissions = Permissions {
    bits: 0b0000_0000_0000_0000_0000_0000_1010_1000,
};

pub const WIKI_DOMAIN: &str = "lotrminecraftmod.fandom.com";

pub const CURSE_API: &str = "https://api.cfwidget.com/minecraft/mc-mods/";
pub const MINECRAFT_API: &str = "https://api.mcsrvstat.us/2/";
pub const GOOGLE_API: &str = "https://www.googleapis.com/customsearch/v1?";

pub const CURSEFORGE_ID_RENEWED: u32 = 406893;
pub const CURSEFORGE_ID_LEGACY: u32 = 423748;

pub const TABLE_PREFIX: &str = "lotr_mod_bot_prefix";
pub const TABLE_ADMINS: &str = "bot_admins";
pub const TABLE_FLOPPA: &str = "floppa_images";
pub const TABLE_USER_BLACKLIST: &str = "user_blacklist";
pub const TABLE_CHANNEL_BLACKLIST: &str = "channel_blacklist";
pub const TABLE_MC_SERVER_IP: &str = "mc_server_ip";
pub const TABLE_CUSTOM_COMMANDS: &str = "custom_commands";

pub const RESERVED_NAMES: [&str; 37] = [
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
];
