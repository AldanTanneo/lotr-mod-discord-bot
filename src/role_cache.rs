use dashmap::DashMap;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use crate::database::roles;

#[derive(Clone, Eq)]
pub struct RoleKey {
    pub name: String,
    pub guild_id: GuildId,
}

impl std::cmp::PartialEq for RoleKey {
    fn eq(&self, other: &RoleKey) -> bool {
        self.name.eq_ignore_ascii_case(&other.name) && self.guild_id == other.guild_id
    }
}

impl std::hash::Hash for RoleKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_ascii_lowercase().hash(state);
        self.guild_id.hash(state);
    }
}

impl std::fmt::Debug for RoleKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?} {:?})", self.name, self.guild_id)
    }
}

impl RoleKey {
    pub fn new(name: &str, guild_id: GuildId) -> Self {
        Self {
            name: name.to_lowercase(),
            guild_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoleCache(Arc<DashMap<RoleKey, Arc<roles::CustomRole>>>);

impl TypeMapKey for RoleCache {
    type Value = Self;
}

impl std::ops::Deref for RoleCache {
    type Target = DashMap<RoleKey, Arc<roles::CustomRole>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for RoleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleCache {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

macro_rules! get_role_cache {
    ($ctx:ident) => {{
        let data_read = $ctx.data.read().await;
        data_read
            .get::<$crate::role_cache::RoleCache>()
            .expect("Expected a role cache in the type map")
            .clone()
    }};
}

pub async fn get_role(
    ctx: &Context,
    server_id: GuildId,
    role_name: String,
) -> Option<Arc<roles::CustomRole>> {
    let role_cache = get_role_cache!(ctx);

    if let Some(role) = role_cache.get(&RoleKey::new(&role_name, server_id)) {
        return Some(role.value().clone());
    }
    if let Some(role) = roles::get_role(ctx, server_id, &role_name).await {
        let r = Arc::new(role);
        role_cache.insert(RoleKey::new(&role_name, server_id), r.clone());
        if let Some(aliases) = roles::get_aliases(ctx, server_id, r.id).await {
            for alias in &aliases {
                role_cache.insert(RoleKey::new(alias, server_id), r.clone());
            }
        }
        Some(r.clone())
    } else {
        None
    }
}

pub async fn add_role(ctx: &Context, server_id: GuildId, role: roles::CustomRole) -> CommandResult {
    roles::add_role(ctx, server_id, &role).await?;

    let role_cache = get_role_cache!(ctx);

    let key = RoleKey::new(&role.name, server_id);

    role_cache.retain(
        |RoleKey {
             name: _,
             guild_id: cached_server_id,
         },
         cached_role| { cached_server_id != &server_id && cached_role.id != role.id },
    );

    let arc_role = Arc::new(role);

    if let Some(aliases) = &arc_role.properties.aliases {
        for alias in aliases {
            role_cache.insert(RoleKey::new(alias, server_id), arc_role.clone());
        }
    }

    role_cache.insert(key, arc_role);

    Ok(())
}

pub async fn delete_role(ctx: &Context, server_id: GuildId, role_id: RoleId) -> CommandResult {
    roles::delete_role(ctx, server_id, role_id).await?;
    let role_cache = get_role_cache!(ctx);

    role_cache.retain(
        |RoleKey {
             name: _,
             guild_id: cached_server_id,
         },
         cached_role| { cached_server_id != &server_id && cached_role.id != role_id },
    );

    Ok(())
}
