//! Module for API functions: queries to google, curseforge, the LOTR Mod wiki...

pub mod curseforge;
pub mod google;
pub mod minecraft;
pub mod wiki;

use serenity::prelude::TypeMapKey;

#[derive(Debug, Clone)]
pub struct ReqwestClient(reqwest::Client);

impl TypeMapKey for ReqwestClient {
    type Value = Self;
}

impl std::ops::Deref for ReqwestClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self(
            reqwest::Client::builder()
                .use_rustls_tls()
                .user_agent("LOTR Mod Discord Bot")
                .build()
                .expect("Could not build the reqwest client"),
        )
    }

    pub fn inner(&self) -> reqwest::Client {
        self.0.clone()
    }
}
