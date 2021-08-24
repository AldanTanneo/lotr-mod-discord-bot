use chrono::{DateTime, NaiveDateTime, Utc};
use serenity::model::prelude::*;
use serenity::utils::Colour;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct CustomCommand {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
}

use BugStatus::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BugStatus {
    Resolved,
    Low,
    Medium,
    High,
    Critical,
    Closed,
    ForgeVanilla,
}

impl std::fmt::Display for BugStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ForgeVanilla => write!(f, "Forge or Vanilla"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseStatusError {}

impl FromStr for BugStatus {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "resolved" => Ok(Resolved),
            "low" => Ok(Low),
            "medium" => Ok(Medium),
            "high" => Ok(High),
            "critical" => Ok(Critical),
            "closed" => Ok(Closed),
            "forgevanilla" | "forge" | "vanilla" => Ok(ForgeVanilla),
            _ => Err(Self::Err {}),
        }
    }
}

impl std::default::Default for BugStatus {
    fn default() -> Self {
        Medium
    }
}

impl BugStatus {
    #[inline]
    pub const fn as_str(&self) -> &str {
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
    pub const fn colour(&self) -> Colour {
        match self {
            Resolved => Colour::new(0x2fd524),
            Low => Colour::new(0xfef01),
            Medium => Colour::new(0xfd9a01),
            High => Colour::new(0xfd6104),
            Critical => Colour::new(0xff0000),
            Closed => Colour::new(0x7694cb),
            ForgeVanilla => Colour::new(0x9f00c5),
        }
    }

    pub const fn marker(&self) -> &str {
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

    pub const fn icon(&self) -> &str {
        match self {
            Resolved => "✅",
            Low | Medium | High | Critical => "⚠️",
            Closed => "❌",
            ForgeVanilla => "🇻", // not a V: the [V] emoji
        }
    }

    pub fn reaction(&self) -> ReactionType {
        ReactionType::Unicode(self.icon().to_string())
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
            status: status.parse().unwrap_or_default(),
            timestamp: DateTime::from_utc(timestamp, Utc),
            legacy,
        })
    }

    pub fn new_from_tuple(data: (u64, String, String, NaiveDateTime, bool)) -> Option<Self> {
        Self::new(data.0, data.1, data.2, data.3, data.4)
    }
}
