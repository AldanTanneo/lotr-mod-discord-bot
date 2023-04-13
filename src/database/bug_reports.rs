use chrono::{DateTime, NaiveDateTime, Utc};
use const_format::formatcp;
use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::utils::Colour;

use crate::constants::{
    TABLE_BUG_REPORTS, TABLE_BUG_REPORTS_LINKS, TABLE_BUG_REPORTS_NOTIFICATIONS,
};
use crate::get_database_conn;

#[derive(Debug, Clone, Copy)]
pub enum BugOrder {
    Chronological(bool),
    Priority(bool),
    None,
}

use BugStatus::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum BugStatus {
    Closed,
    ForgeVanilla,
    Resolved,
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for BugStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ForgeVanilla => write!(f, "Forge or Vanilla"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseStatusError;

impl std::str::FromStr for BugStatus {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "resolved" => Resolved,
            "low" => Low,
            "medium" => Medium,
            "high" => High,
            "critical" => Critical,
            "closed" => Closed,
            "forgevanilla" | "forge" | "vanilla" => ForgeVanilla,
            _ => return Err(Self::Err {}),
        })
    }
}

impl BugStatus {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Resolved => "resolved",
            Low => "low",
            Medium => "medium",
            High => "high",
            Critical => "critical",
            Closed => "closed",
            ForgeVanilla => "forgevanilla",
        }
    }

    #[inline]
    pub const fn colour(self) -> Colour {
        match self {
            Resolved => Colour(0x2fd524),
            Low => Colour(0xfef001),
            Medium => Colour(0xfd9a01),
            High => Colour(0xfd6104),
            Critical => Colour(0xff0000),
            Closed => Colour(0x7694cb),
            ForgeVanilla => Colour(0x9f00c5),
        }
    }

    pub const fn marker(self) -> &'static str {
        match self {
            Resolved => ":green_circle:",
            Low => ":yellow_circle:",
            Medium => ":orange_circle:",
            High => ":red_circle:",
            Critical => ":bangbang:",
            Closed => ":blue_circle:",
            ForgeVanilla => ":regional_indicator_v:",
        }
    }

    pub fn reaction(self) -> ReactionType {
        ReactionType::Unicode(
            match self {
                Resolved => "✅",
                Low | Medium | High | Critical => "⚠️",
                Closed => "❌",
                ForgeVanilla => "🇻", // not a V: the [V] emoji
            }
            .to_string(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BugLink {
    pub id: u64,
    pub url: String,
    pub title: String,
}

impl std::fmt::Display for BugLink {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]({}) (#{})", self.title, self.url, self.id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum BugCategory {
    #[default]
    Renewed,
    Legacy,
}

#[derive(Debug, Clone)]
pub struct ParseCategoryError;

impl std::str::FromStr for BugCategory {
    type Err = ParseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BugCategory::*;

        Ok(match s.to_ascii_lowercase().as_str() {
            "renewed" => Renewed,
            "legacy" => Legacy,
            _ => return Err(ParseCategoryError),
        })
    }
}

impl std::fmt::Display for BugCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl BugCategory {
    pub const fn as_str(self) -> &'static str {
        use BugCategory::*;
        match self {
            Renewed => "renewed",
            Legacy => "legacy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BugReport {
    pub bug_id: u64,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub title: String,
    pub status: BugStatus,
    pub timestamp: DateTime<Utc>,
    pub category: BugCategory,
    pub links: Vec<BugLink>,
}

#[derive(Debug, Clone)]
pub struct PartialBugReport {
    pub bug_id: u64,
    pub title: String,
    pub status: BugStatus,
    pub timestamp: DateTime<Utc>,
    pub category: BugCategory,
}

impl std::fmt::Display for PartialBugReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = Utc::now().signed_duration_since(self.timestamp).num_days();
        let format_str = match duration {
            0..=6 => "<t:%s:R>",
            _ => "<t:%s:d>",
        };
        write!(
            f,
            "LOTR-{} — {}  ({})",
            self.bug_id,
            self.title,
            self.timestamp.format(format_str)
        )
    }
}

impl PartialBugReport {
    pub fn new(
        bug_id: u64,
        title: String,
        status: BugStatus,
        timestamp: NaiveDateTime,
        category: BugCategory,
    ) -> Option<Self> {
        Some(Self {
            bug_id,
            title,
            status,
            timestamp: DateTime::from_utc(timestamp, Utc),
            category,
        })
    }
}

pub async fn get_bug_from_id(ctx: &Context, bug_id: u64) -> Result<BugReport, CommandError> {
    let mut conn = get_database_conn!(ctx);

    let (channel_id, message_id, title, status, timestamp, category): (
        u64,
        u64,
        String,
        String,
        NaiveDateTime,
        String,
    ) = conn
        .exec_first(
            formatcp!(
                "SELECT channel_id, message_id, title, status, timestamp, category \
FROM {} WHERE bug_id = :bug_id",
                TABLE_BUG_REPORTS
            ),
            params! {
                "bug_id" => bug_id
            },
        )
        .await?
        .ok_or_else(|| CommandError::from("Bug report does not exist!"))?;

    let links: Vec<BugLink> = conn
        .exec_map(
            formatcp!(
                "SELECT link_id, link_url, link_title FROM {} WHERE bug_id = :bug_id",
                TABLE_BUG_REPORTS_LINKS
            ),
            params! {
                "bug_id" => bug_id
            },
            |(id, url, title)| BugLink { id, url, title },
        )
        .await?;

    Ok(BugReport {
        bug_id,
        channel_id: ChannelId(channel_id),
        message_id: MessageId(message_id),
        title,
        status: status
            .parse()
            .expect("Expected a valid bug status from the database"),
        timestamp: DateTime::from_utc(timestamp, Utc),
        category: category
            .parse()
            .expect("Expected a valid category from the database"),
        links,
    })
}

pub async fn add_bug_report(
    ctx: &Context,
    msg: &Message,
    title: String,
    status: BugStatus,
    category: BugCategory,
) -> Result<u64, CommandError> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        formatcp!(
            "INSERT INTO {} (channel_id, message_id, title, status, category) \
VALUES (:channel_id, :message_id, :title, :status, :category)",
            TABLE_BUG_REPORTS
        ),
        params! {
            "channel_id" => msg.channel_id.0,
            "message_id" => msg.id.0,
            "title" => title,
            "status" => status.as_str(),
            "category" => category.as_str(),
        },
    )
    .await?;

    if let Err(e) = msg.react(ctx, status.reaction()).await {
        println!("Could not add reaction to bug report: {}", e);
    }

    conn.query_first(formatcp!("SELECT MAX(bug_id) FROM {}", TABLE_BUG_REPORTS))
        .await?
        .ok_or_else(|| CommandError::from("Could not get newest bug id!"))
}

pub async fn get_bug_list(
    ctx: &Context,
    status: Option<BugStatus>,
    limit: u32,
    display_order: BugOrder,
    category: Option<BugCategory>,
    page: u32,
) -> Option<(Vec<PartialBugReport>, u32)> {
    let mut conn = get_database_conn!(ctx);

    let total: u32 = conn
        .query_first(format!(
            "SELECT COUNT(bug_id) FROM {} WHERE {} {category}",
            TABLE_BUG_REPORTS,
            if let Some(status) = status {
                format!("status = '{}'", status.as_str())
            } else {
                "status != 'resolved' AND status != 'closed' AND status != 'forgevanilla'".into()
            },
            category = if let Some(c) = category {
                format!("AND category = '{}'", c.as_str())
            } else {
                "".into()
            },
        ))
        .await
        .ok()??;

    conn.exec_map(
        format!(
            "SELECT bug_id, title, status, timestamp, category FROM {} \
WHERE {} {category} ORDER BY {ordering} LIMIT :limit OFFSET :offset",
            TABLE_BUG_REPORTS,
            if let Some(status) = status {
                format!("status = '{}'", status.as_str())
            } else {
                "status != 'resolved' AND status != 'closed' AND status != 'forgevanilla'".into()
            },
            category = if let Some(c) = category {
                format!("AND category = '{}'", c.as_str())
            } else {
                "".into()
            },
            ordering = match display_order {
                BugOrder::Chronological(false) | BugOrder::None => "timestamp DESC",
                BugOrder::Chronological(true) => "timestamp ASC",
                BugOrder::Priority(false) => "status DESC, timestamp DESC",
                BugOrder::Priority(true) => "status ASC, timestamp DESC",
            },
        ),
        params! {
            "limit" => limit,
            "offset" => limit * page
        },
        |(bug_id, title, status, timestamp, category): (
            u64,
            String,
            String,
            NaiveDateTime,
            String,
        )| {
            PartialBugReport::new(
                bug_id,
                title,
                status
                    .parse::<BugStatus>()
                    .expect("Expected a valid bug status from the database"),
                timestamp,
                category
                    .parse()
                    .expect("Expected a valid category from the database"),
            )
        },
    )
    .await
    .ok()
    .map(|v| v.into_iter().flatten().collect())
    .map(|v| (v, total))
}

pub async fn change_bug_status(
    ctx: &Context,
    bug_id: u64,
    new_status: BugStatus,
) -> Result<BugStatus, CommandError> {
    let mut conn = get_database_conn!(ctx);

    let (old_status_string, channel_id, msg_id): (String, u64, u64) = conn
        .exec_first(
            formatcp!(
                "SELECT status, channel_id, message_id FROM {} WHERE bug_id = :bug_id LIMIT 1",
                TABLE_BUG_REPORTS
            ),
            params! {
                "bug_id" => bug_id
            },
        )
        .await?
        .ok_or_else(|| CommandError::from("Could not find bug in database"))?;

    let old_status: BugStatus = old_status_string
        .parse()
        .expect("Expected a valid bug status from database!");

    conn.exec_drop(
        formatcp!(
            "UPDATE {} SET status = :status WHERE bug_id = :bug_id",
            TABLE_BUG_REPORTS
        ),
        params! {
            "status" => new_status.as_str(),
            "bug_id" => bug_id
        },
    )
    .await?;

    match ChannelId(channel_id).message(ctx, MessageId(msg_id)).await {
        Ok(msg) => {
            if let Err(e) = msg.delete_reaction_emoji(ctx, old_status.reaction()).await {
                println!("Could not remove reaction from bug report: {}", e);
            }
            if let Err(e) = msg.react(ctx, new_status.reaction()).await {
                println!("Could not add reaction to bug report: {}", e);
            }
        }
        Err(e) => println!("Could not get message for bug report: {}", e),
    }

    Ok(old_status)
}

pub async fn add_link(ctx: &Context, bug_id: u64, link_url: &str, link_title: &str) -> Option<u64> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        formatcp!(
            "INSERT INTO {} (bug_id, link_url, link_title) VALUES (:bug_id, :link_url, :link_title)",
            TABLE_BUG_REPORTS_LINKS
        ),
        params! {
            "bug_id" => bug_id,
            "link_url" => link_url,
            "link_title" => link_title
        },
    )
    .await.ok()?;

    conn.exec_first(
        formatcp!(
            "SELECT MAX(link_id) FROM {} WHERE bug_id = :bug_id",
            TABLE_BUG_REPORTS_LINKS
        ),
        params! {
            "bug_id" => bug_id
        },
    )
    .await
    .ok()?
}

pub async fn remove_link(ctx: &Context, bug_id: u64, link_num: u64) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        formatcp!(
            "DELETE FROM {} WHERE bug_id = :bug_id AND link_id = :link_id",
            TABLE_BUG_REPORTS_LINKS
        ),
        params! {
            "bug_id" => bug_id,
            "link_id" => link_num
        },
    )
    .await?;

    Ok(())
}

pub async fn change_title(ctx: &Context, bug_id: u64, new_title: &str) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    conn.exec_drop(
        formatcp!(
            "UPDATE {} SET title = :new_title WHERE bug_id = :bug_id",
            TABLE_BUG_REPORTS
        ),
        params! {
            "new_title" => new_title,
            "bug_id" => bug_id
        },
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct BugCounts {
    pub resolved: u32,
    pub low: u32,
    pub medium: u32,
    pub high: u32,
    pub critical: u32,
    pub closed: u32,
    pub forgevanilla: u32,
    pub total: u32,
    pub legacy: u32,
}

pub async fn get_bug_statistics(ctx: &Context) -> Option<BugCounts> {
    let mut conn = get_database_conn!(ctx);

    let statuses = [
        "resolved",
        "low",
        "medium",
        "high",
        "critical",
        "closed",
        "forgevanilla",
    ];

    let mut counts = [0; 9];

    for (i, s) in statuses.iter().enumerate() {
        let x = conn
            .exec_first(
                formatcp!(
                    "SELECT COUNT(bug_id) FROM {} WHERE status = :status",
                    TABLE_BUG_REPORTS
                ),
                params! {
                    "status" => s
                },
            )
            .await
            .ok()??;
        counts[i] = x;
    }

    counts[7] = counts.iter().sum();

    counts[8] = conn
        .query_first(formatcp!(
            "SELECT COUNT(bug_id) FROM {} WHERE category = 'legacy'",
            TABLE_BUG_REPORTS
        ))
        .await
        .ok()??;

    Some(BugCounts {
        resolved: counts[0],
        low: counts[1],
        medium: counts[2],
        high: counts[3],
        critical: counts[4],
        closed: counts[5],
        forgevanilla: counts[6],
        total: counts[7],
        legacy: counts[8],
    })
}

pub async fn change_category(
    ctx: &Context,
    bug_id: u64,
    category: BugCategory,
) -> Option<BugCategory> {
    let mut conn = get_database_conn!(ctx);

    let old_category = conn
        .exec_first::<String, _, _>(
            formatcp!(
                "SELECT category FROM {} WHERE bug_id = :bug_id",
                TABLE_BUG_REPORTS
            ),
            params! {
                "bug_id" => bug_id,
            },
        )
        .await
        .ok()??
        .parse()
        .expect("Expected a valid bug category from the database");

    conn.exec_drop(
        formatcp!(
            "UPDATE {} SET category = :category WHERE bug_id = :bug_id",
            TABLE_BUG_REPORTS
        ),
        params! {
            "category" => category.as_str(),
            "bug_id" => bug_id
        },
    )
    .await
    .ok()?;

    Some(old_category)
}

pub async fn is_notified_user(ctx: &Context, bug_id: u64, user_id: UserId) -> Option<bool> {
    let mut conn = get_database_conn!(ctx);

    conn.exec_first(
        formatcp!(
            "SELECT EXISTS(SELECT notification_id \
FROM {} WHERE bug_id = :bug_id AND user_id = :user_id)",
            TABLE_BUG_REPORTS_NOTIFICATIONS
        ),
        params! {
                "bug_id" => bug_id,
                "user_id" => user_id.0
        },
    )
    .await
    .ok()?
}

pub async fn get_notifications_for_user(
    ctx: &Context,
    user_id: UserId,
    closed: bool,
) -> CommandResult<Vec<u64>> {
    let mut conn = get_database_conn!(ctx);

    Ok(if closed {
        conn.exec(
            formatcp!(
                "SELECT bug_id FROM {} WHERE user_id = :user_id",
                TABLE_BUG_REPORTS_NOTIFICATIONS,
            ),
            params! {
                "user_id" => user_id.0
            },
        )
        .await?
    } else {
        conn.exec(
            formatcp!(
                "SELECT t1.bug_id FROM {TABLE_BUG_REPORTS_NOTIFICATIONS} AS t1 \
JOIN {TABLE_BUG_REPORTS} AS t2 \
ON t1.bug_id = t2.bug_id \
AND t2.status != 'closed' \
AND t2.status != 'resolved' \
AND t2.status != 'forgevanilla' \
WHERE t1.user_id = :user_id"
            ),
            params! {"user_id" => user_id.0},
        )
        .await?
    })
}

pub async fn get_notified_users(ctx: &Context, bug_id: u64) -> CommandResult<Vec<UserId>> {
    let mut conn = get_database_conn!(ctx);

    Ok(conn
        .exec_map(
            formatcp!(
                "SELECT user_id FROM {} WHERE bug_id = :bug_id",
                TABLE_BUG_REPORTS_NOTIFICATIONS
            ),
            params! {
                "bug_id" => bug_id
            },
            UserId,
        )
        .await?)
}

pub async fn add_notified_user(ctx: &Context, bug_id: u64, user_id: UserId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    Ok(conn
        .exec_drop(
            formatcp!(
                "INSERT INTO {} (bug_id, user_id) VALUES (:bug_id, :user_id)",
                TABLE_BUG_REPORTS_NOTIFICATIONS
            ),
            params! {
                "bug_id" => bug_id,
                "user_id" => user_id.0
            },
        )
        .await?)
}

pub async fn remove_notified_user(ctx: &Context, bug_id: u64, user_id: UserId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);

    Ok(conn
        .exec_drop(
            formatcp!(
                "DELETE FROM {} WHERE bug_id = :bug_id AND user_id = :user_id LIMIT 1",
                TABLE_BUG_REPORTS_NOTIFICATIONS
            ),
            params! {
                "bug_id" => bug_id,
                "user_id" => user_id.0
            },
        )
        .await?)
}
