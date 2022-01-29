//! Constants used in commands and API bindings

use poise::serenity::model::prelude::*;

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

pub mod socials {

    pub mod facebook {
        use poise::serenity::utils::Colour;

        /// Mod's facebook page url
        pub const URL: &str = "https://www.facebook.com/LOTRMC";

        /// Facebook logo for the /facebook command
        pub const ICON: &str =
            "https://facebookbrand.com/wp-content/uploads/2019/04/f_logo_RGB-Hex-Blue_512.png";
        /// Facebook embed colour
        pub const COLOUR: Colour = Colour(0x1877f2);
    }

    pub mod instagram {
        use poise::serenity::utils::Colour;

        /// Mod's instagram page url
        pub const URL: &str = "https://www.instagram.com/lotrmcmod/";

        /// Instagram logo for the /instagram command
        pub const ICON: &str =
    "https://upload.wikimedia.org/wikipedia/commons/thumb/e/e7/Instagram_logo_2016.svg/768px-Instagram_logo_2016.svg.png";
        /// Instagram embed colour
        pub const COLOUR: Colour = Colour(0xc13584);
    }
}

pub mod donate {
    use poise::serenity::utils::Colour;

    /// Visual for the !donate command
    pub const ICON: &str =
        "https://media.discordapp.net/attachments/781837314975989772/809773869971013653/Donate.png";

    /// Colour for the /donate embed
    pub const COLOUR: Colour = Colour(0xCEBD9C);

    /// Paypal donation link in dollars
    pub const URL_DOLLARS: &str =
        "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=YZ97X6UBJJD7Y";
    /// Paypal donation link in pounds
    pub const URL_POUNDS: &str =
        "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=8BXR2F4FYYEK2";
    /// Paypal donation link in euros
    pub const URL_EUROS: &str =
        "https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=5Q4NK7C5N2FB4";
}
