pub mod admin_data;
pub mod blacklist;
pub mod bug_reports;
pub mod config;
pub mod custom_commands;
pub mod floppa;
pub mod maintenance;

use mysql_async::Pool;
use serenity::model::prelude::*;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use Blacklist::*;

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}

pub enum Blacklist {
    IsBlacklisted(bool),
    List(Vec<UserId>, Vec<ChannelId>),
}

impl Blacklist {
    pub fn is_blacklisted(&self) -> bool {
        match self {
            IsBlacklisted(b) => *b,
            _ => false,
        }
    }

    pub fn get_list(&self) -> (Vec<UserId>, Vec<ChannelId>) {
        match self {
            List(a, b) => (a.to_vec(), b.to_vec()),
            _ => (vec![], vec![]),
        }
    }
}

pub struct CustomCommand {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
}
