use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::*;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;

pub use super::{BugReport, BugStatus, PartialBugReport};
use crate::constants::{TABLE_BUG_REPORTS, TABLE_BUG_REPORTS_LINKS};
use crate::get_database_conn;

pub async fn get_bug_from_id(ctx: &Context, bug_id: u64) -> Option<BugReport> {
    let mut conn;
    get_database_conn!(ctx, conn);

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
        status: status.as_str().parse().unwrap_or_default(),
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
    let mut conn;
    get_database_conn!(ctx, conn);

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
    page: u32,
) -> Option<(Vec<PartialBugReport>, u32)> {
    let mut conn;
    get_database_conn!(ctx, conn);

    let total: u32 = conn
        .query_first(format!(
            "SELECT COUNT(bug_id) FROM {} WHERE {} {legacy}",
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
        ))
        .await
        .ok()??;

    conn.exec_map(
        format!(
            "SELECT bug_id, title, status, timestamp, legacy FROM {} WHERE {} {legacy} ORDER BY timestamp {order} LIMIT :limit OFFSET :offset",
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
            "offset" => limit * page
        },
        PartialBugReport::new_from_tuple,
    )
    .await
    .ok()
    .map(|v| v.iter().filter_map(|x| x.clone()).collect()).map(|v| (v, total))
}

pub async fn change_bug_status(
    ctx: &Context,
    bug_id: u64,
    new_status: BugStatus,
) -> Option<BugStatus> {
    let mut conn;
    get_database_conn!(ctx, conn);

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

    Some(old_status.parse().unwrap_or_default())
}

pub async fn add_link(ctx: &Context, bug_id: u64, link_url: &str, link_title: &str) -> Option<u64> {
    let mut conn;
    get_database_conn!(ctx, conn);

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
    let mut conn;
    get_database_conn!(ctx, conn, Result);

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
    let mut conn;
    get_database_conn!(ctx, conn, Result);

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

pub async fn get_bug_statistics(ctx: &Context) -> Option<[u32; 8]> {
    let mut conn;
    get_database_conn!(ctx, conn);

    let statuses = ["resolved", "low", "medium", "high", "critical", "closed"];

    let mut counts = [0; 8];

    for (i, s) in statuses.iter().enumerate() {
        let x = conn
            .query_first(format!(
                "SELECT COUNT(bug_id) FROM {} WHERE status = '{}'",
                TABLE_BUG_REPORTS, s
            ))
            .await
            .ok()??;
        counts[i] = x;
    }

    counts[6] = counts.iter().sum();

    counts[7] = conn
        .query_first(format!(
            "SELECT COUNT(bug_id) FROM {} WHERE legacy = 1",
            TABLE_BUG_REPORTS
        ))
        .await
        .ok()??;

    Some(counts)
}
