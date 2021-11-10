//! Module for database interactions

pub mod admin_data;
pub mod blacklist;
pub mod bug_reports;
pub mod config;
pub mod custom_commands;
pub mod floppa;
pub mod maintenance;
pub mod qa_data;
pub mod roles;

use mysql_async::{OptsBuilder, Pool};
use serenity::prelude::TypeMapKey;

#[derive(Debug, Clone)]
pub struct DatabasePool(Pool);

impl TypeMapKey for DatabasePool {
    type Value = Self;
}

impl std::ops::Deref for DatabasePool {
    type Target = Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DatabasePool {
    pub fn new(opts: OptsBuilder) -> Self {
        Self(Pool::new(opts))
    }
}
