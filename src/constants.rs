//! Constants used in commands and API bindings
use serenity::model::prelude::*;
use serenity::utils::Colour;

/// User ID of the bot
pub const BOT_ID: UserId = UserId(780858391383638057);
/// User ID of the owner
pub const OWNER_ID: UserId = UserId(405421991777009678);
/// Guild ID of the [LOTR Mod Community Discord](https://discord.gg/QXkZzKU)
pub const LOTR_DISCORD: GuildId = GuildId(405091134327619587);

/// Maximum size, in bytes, of a JSON file for [announcements][crate::announcement]
/// and [custom commands][crate::commands::custom_commands]
pub const MAX_JSON_FILE_SIZE: u64 = 10240;

/// Bit filter for colours
pub const BIT_FILTER_24BITS: u32 = (1 << 24) - 1;

/// Set of [permissions][Permissions] needed to manage the bot (bot admins and owner excepted).
pub const MANAGE_BOT_PERMS: Permissions = Permissions::ADMINISTRATOR
    .union(Permissions::MANAGE_GUILD)
    .union(Permissions::MANAGE_CHANNELS)
    .union(Permissions::MANAGE_ROLES)
    .union(Permissions::MANAGE_WEBHOOKS)
    .union(Permissions::MODERATE_MEMBERS);

/// Set of [permissions][Permissions] needed for the bot when inviting it to a server
pub const INVITE_PERMISSIONS: Permissions = Permissions::MANAGE_ROLES
    .union(Permissions::CHANGE_NICKNAME)
    .union(Permissions::VIEW_CHANNEL)
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

/// Bot icon
pub const BOT_ICON: &str =
    "https://cdn.discordapp.com/avatars/780858391383638057/ed9b9a5b688dc3b8d9ea580584d25033.webp";
/// Termite icon for bug reports
pub const TERMITE_IMAGE: &str =
    "https://media.discordapp.net/attachments/781837314975989772/839479742457839646/termite.png";
/// Forge logo for the !forge command
pub const FORGE_ICON: &str =
    "https://pbs.twimg.com/profile_images/778706890914095109/fhMDH9o6_normal.jpg";
/// Curseforge logo for the !curseforge command
pub const CURSEFORGE_ICON: &str =
    "https://cdn-images-1.medium.com/v2/resize:fill:45:45/1*ZCi5iwccyX6AYBt4pjY_BQ.png";

/// Facebook logo for the !facebook command
pub const FACEBOOK_ICON: &str =
    "https://facebookbrand.com/wp-content/uploads/2019/04/f_logo_RGB-Hex-Blue_512.png";
/// Facebook embed colour
pub const FACEBOOK_COLOUR: Colour = Colour::new(0x1877f2);

/// Instagram logo for the !instagram command
pub const INSTAGRAM_ICON: &str =
    "https://upload.wikimedia.org/wikipedia/commons/thumb/e/e7/Instagram_logo_2016.svg/768px-Instagram_logo_2016.svg.png";
/// Instagram embed colour
pub const INSTAGRAM_COLOUR: Colour = Colour::new(0xC13584);

/// Visual for the !donate command
pub const DONATE_THUMBNAIL: &str =
    "https://media.discordapp.net/attachments/781837314975989772/809773869971013653/Donate.png";

/// Link to forge downloads for the !forge command. Replace {mc} with the correct version.
pub const FORGE_LINK: &str =
    "http://files.minecraftforge.net/maven/net/minecraftforge/forge/index_{mc}.html";
/// [LOTR Mod Wiki](https://lotrminecraftmod.fandom.com) adress
pub const WIKI_DOMAIN: &str = "lotrminecraftmod.fandom.com";
/// Paypal donation link in dollars
pub const PAYPAL_LINK_DOLLARS: &str =
    "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=YZ97X6UBJJD7Y";
/// Paypal donation link in pounds
pub const PAYPAL_LINK_POUNDS: &str =
    "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=8BXR2F4FYYEK2";
/// Paypal donation link in euros
pub const PAYPAL_LINK_EUROS: &str =
    "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=5Q4NK7C5N2FB4";

/// The [Curseforge API](https://www.curseforge.com/) for the
/// [`!curseforge`][crate::commands::general::curseforge] command
pub const CURSE_API: &str = "https://api.curseforge.com/v1/mods/";
/// A Minecraft server [public API](https://api.mcstatus.io/) for the [`!online`][crate::commands::servers::online] command
pub const MINECRAFT_API: &str = "https://api.mcstatus.io/v2/status/java/";
/// Google API for custom google search
pub const GOOGLE_API: &str = "https://www.googleapis.com/customsearch/v1?";

/// Curseforge project ID for the [LOTR Mod Renewed](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-renewed)
pub const CURSEFORGE_ID_RENEWED: u64 = 406893;
/// Curseforge project ID for the [LOTR Mod Legacy](https://www.curseforge.com/minecraft/mc-mods/the-lord-of-the-rings-mod-legacy)
pub const CURSEFORGE_ID_LEGACY: u64 = 423748;

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
/// SQL table name for [bug report notifications][crate::database::bug_reports]
pub const TABLE_BUG_REPORTS_NOTIFICATIONS: &str = "bug_reports__notifications";
/// SQL table name for [role handling][crate::database::roles]
pub const TABLE_ROLES: &str = "roles";
/// SQL table name for [role aliases handling][crate::database::roles]
pub const TABLE_ROLES_ALIASES: &str = "roles__aliases";
/// SQL table name for guild list and database cleanup
pub const TABLE_LIST_GUILDS: &str = "list_guilds";

/// Reserved command names that cannot be used as [custom commands][crate::commands::custom_commands]
pub const RESERVED_NAMES: [&str; 52] = [
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
    "role",
    "roles",
    "listroles",
    "instagram",
    "ig",
    "q&a",
    "shutdown",
];
