use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::utils::Colour;

use super::DatabasePool;
use crate::constants::{TABLE_BUG_REPORTS, TABLE_BUG_REPORTS_LINKS};

use BugStatus::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BugStatus {
    Resolved,
    Low,
    Medium,
    High,
    Critical,
    Closed,
}

impl<'a, S: Into<&'a str>> From<S> for BugStatus {
    fn from(s: S) -> Self {
        match s.into().to_lowercase().as_str() {
            "resolved" => Resolved,
            "low" => Low,
            "medium" => Medium,
            "high" => High,
            "critical" => Critical,
            "closed" => Closed,
            _ => Medium,
        }
    }
}

impl<'a> BugStatus {
    pub fn as_str(&self) -> &'a str {
        match self {
            Resolved => "resolved",
            Low => "low",
            Medium => "medium",
            High => "high",
            Critical => "critical",
            Closed => "closed",
        }
    }

    pub fn colour(&self) -> Colour {
        match self {
            Resolved => Colour::FOOYOO,
            Low => Colour::KERBAL,
            Medium => Colour::GOLD,
            High => Colour::ORANGE,
            Critical => Colour::RED,
            Closed => Colour::FABLED_PINK,
        }
    }

    pub fn icon(&self) -> &'a str {
        match self {
            Resolved => "✅",
            Low | Medium | High | Critical => "⚠️",
            Closed => "❌",
        }
    }
}

pub struct BugReport {
    pub bug_id: u64,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub title: String,
    pub status: BugStatus,
    pub timestamp: DateTime<Utc>,
    pub legacy: bool,
    pub links: Vec<(u64, String, String)>,
}

#[derive(Debug, Clone)]
pub struct PartialBugReport {
    pub bug_id: u64,
    pub title: String,
    pub status: BugStatus,
    pub timestamp: DateTime<Utc>,
    pub legacy: bool,
}

impl std::fmt::Display for PartialBugReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_minutes();
        let tmp = format!("{}mins ago", duration + 1);
        let format_str = match duration {
            0..=59 => tmp.as_str(),
            60..=1439 => "Today at %R",
            1440..=2879 => "Yesterday at %R",
            _ => "%d/%m/%Y",
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
    fn new(
        bug_id: u64,
        title: String,
        status: String,
        timestamp: NaiveDateTime,
        legacy: bool,
    ) -> Option<Self> {
        Some(Self {
            bug_id,
            title,
            status: status.as_str().into(),
            timestamp: DateTime::from_utc(timestamp, Utc),
            legacy,
        })
    }

    fn new_from_tuple(data: (u64, String, String, NaiveDateTime, bool)) -> Option<Self> {
        Self::new(data.0, data.1, data.2, data.3, data.4)
    }
}

pub async fn get_bug_from_id(ctx: &Context, bug_id: u64) -> Option<BugReport> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    let (channel_id, message_id, title, status, timestamp, legacy): (
        u64,
        u64,
        String,
        String,
        NaiveDateTime,
        bool,
    ) = conn
        .query_first(format!(
            "SELECT channel_id, message_id, title, status, timestamp, legacy FROM {} WHERE bug_id={}",
            TABLE_BUG_REPORTS, bug_id
        ))
        .await
        .ok()??;

    let links: Vec<(u64, String, String)> = dbg!(
        conn.exec(
            format!(
                "SELECT link_id, link_url, link_title FROM {} WHERE bug_id = :bug_id",
                TABLE_BUG_REPORTS_LINKS
            ),
            params! {
                "bug_id" => bug_id
            },
        )
        .await
    )
    .ok()?;

    Some(BugReport {
        bug_id,
        channel_id: ChannelId(channel_id),
        message_id: MessageId(message_id),
        title,
        status: status.as_str().into(),
        timestamp: DateTime::from_utc(timestamp, Utc),
        legacy,
        links,
    })
}

pub async fn add_bug_report(
    ctx: &Context,
    msg: &Message,
    title: String,
    status: BugStatus,
    legacy: bool,
) -> Option<u64> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    conn.exec_drop(
        format!(
            "INSERT INTO {} (channel_id, message_id, title, status, legacy) VALUES (:channel_id, :message_id, :title, :status, :legacy)",
            TABLE_BUG_REPORTS
        ),
        params! {
            "channel_id" => msg.channel_id.0,
            "message_id" => msg.id.0,
            "title" => title,
            "status" => status.as_str(),
            "legacy" => legacy,
        }
    ).await.ok()?;

    conn.query_first(format!("SELECT MAX(bug_id) FROM {}", TABLE_BUG_REPORTS))
        .await
        .ok()?
}

pub async fn get_bug_list(
    ctx: &Context,
    status: Option<BugStatus>,
    limit: u32,
    ascending: bool,
    legacy: Option<bool>,
) -> Option<Vec<PartialBugReport>> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    conn.exec_map(
        format!(
            "SELECT bug_id, title, status, timestamp, legacy FROM {} WHERE {} {legacy} ORDER BY timestamp {order} LIMIT :limit",
            TABLE_BUG_REPORTS,
            if let Some(status) = status {
                format!("status = '{}'", status.as_str())
            } else {
                "status != 'resolved' AND status != 'closed'".into()
            },
            legacy = if let Some(b) = legacy {
                format!("AND legacy = {}", b as u8)
            } else {
                "".into()
            },
            order = if ascending { "ASC" } else { "DESC" },
        ),
        params! {
            "limit" => limit,
        },
        PartialBugReport::new_from_tuple,
    )
    .await
    .ok()
    .map(|v| v.iter().filter_map(|x| x.clone()).collect())
}

pub async fn change_bug_status(
    ctx: &Context,
    bug_id: u64,
    new_status: BugStatus,
) -> Option<BugStatus> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;
    let old_status: String = conn
        .query_first(format!(
            "SELECT status FROM {} WHERE bug_id = {} LIMIT 1",
            TABLE_BUG_REPORTS, bug_id
        ))
        .await
        .ok()??;

    conn.exec_drop(
        format!(
            "UPDATE {} SET status = :status WHERE bug_id = :bug_id",
            TABLE_BUG_REPORTS
        ),
        params! {
            "status" => new_status.as_str(),
            "bug_id" => bug_id
        },
    )
    .await
    .ok()?;

    Some(old_status.as_str().into())
}

pub async fn add_link(ctx: &Context, bug_id: u64, link_url: &str, link_title: &str) -> Option<u64> {
    let pool = {
        let data_read = ctx.data.read().await;
        data_read.get::<DatabasePool>()?.clone()
    };
    let mut conn = pool.get_conn().await.ok()?;

    conn.exec_drop(
        format!(
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

    conn.query_first(format!(
        "SELECT MAX(link_id) FROM {} WHERE bug_id = {}",
        TABLE_BUG_REPORTS_LINKS, bug_id
    ))
    .await
    .ok()?
}

pub async fn remove_link(ctx: &Context, bug_id: u64, link_num: u64) -> CommandResult {
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not get database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    conn.exec_drop(
        format!(
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
    let pool = {
        let data_read = ctx.data.read().await;
        if let Some(p) = data_read.get::<DatabasePool>() {
            p.clone()
        } else {
            println!("Could not get database pool");
            return Ok(());
        }
    };
    let mut conn = pool.get_conn().await?;

    conn.exec_drop(
        format!(
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
