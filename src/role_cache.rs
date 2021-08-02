use dashmap::DashMap;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

use crate::database::roles;

pub struct RoleCache;

impl TypeMapKey for RoleCache {
    type Value = Arc<DashMap<(String, GuildId), Arc<roles::Role>>>;
}

macro_rules! get_role_cache {
    ($ctx:ident) => {{
        let data_read = $ctx.data.read().await;
        data_read.get::<$crate::role_cache::RoleCache>()?.clone()
    }};
    ($ctx:ident, Result) => {{
        let data_read = $ctx.data.read().await;
        if let Some(cache) = data_read.get::<$crate::role_cache::RoleCache>() {
            cache.clone()
        } else {
            println!("Could not get role cache");
            return Ok(());
        }
    }};
}

pub async fn get_role(
    ctx: &Context,
    server_id: GuildId,
    role_name: String,
) -> Option<Arc<roles::Role>> {
    let role_cache = get_role_cache!(ctx);

    if let Some(role) = role_cache.get(&(role_name.clone(), server_id)) {
        return Some(role.value().clone());
    }
    if let Some(role) = roles::get_role(ctx, server_id, &role_name).await {
        let r = Arc::new(role);
        role_cache.insert((role_name, server_id), r.clone());
        if let Some(aliases) = roles::get_aliases(ctx, server_id, r.id.into()).await {
            for alias in aliases {
                role_cache.insert((alias, server_id), r.clone());
            }
        }
        Some(r.clone())
    } else {
        None
    }
}

pub async fn add_role(ctx: &Context, server_id: GuildId, role: roles::Role) -> CommandResult {
    roles::add_role(ctx, server_id, &role).await?;

    let role_cache = get_role_cache!(ctx, Result);

    let key = (role.name.clone(), server_id);

    role_cache.retain(|(_, cached_server_id), cached_role| {
        cached_server_id != &server_id && cached_role.id != role.id
    });

    let arc_role = Arc::new(role);

    if let Some(aliases) = &arc_role.properties.aliases {
        for alias in aliases.clone() {
            role_cache.insert((alias, server_id), arc_role.clone());
        }
    }

    role_cache.insert(key, arc_role);

    Ok(())
}

pub async fn delete_role(ctx: &Context, server_id: GuildId, role_id: RoleId) -> CommandResult {
    roles::delete_role(ctx, server_id, role_id).await?;
    let role_cache = get_role_cache!(ctx, Result);

    role_cache.retain(|(_, cached_server_id), cached_role| {
        cached_server_id != &server_id && cached_role.id != role_id
    });

    Ok(())
}
