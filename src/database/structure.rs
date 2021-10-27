use sqlx::types::chrono;

#[derive(sqlx::FromRow)]
pub struct BugReport {
    pub bug_id: u32,
    pub channel_id: u64,
    pub message_id: u64,
    pub title: String,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub legacy: bool,
}
