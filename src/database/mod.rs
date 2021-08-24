pub mod admin_data;
pub mod blacklist;
pub mod bug_reports;
pub mod config;
pub mod custom_commands;
pub mod floppa;
pub mod maintenance;
pub mod roles;
pub mod structures;

use mysql_async::Pool;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub use structures::*;

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = Arc<Pool>;
}
