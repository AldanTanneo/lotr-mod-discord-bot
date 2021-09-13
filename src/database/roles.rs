use mysql_async::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::utils::Colour;
use std::iter;
use std::time::Duration;

use crate::commands::roles::format_role_name;
use crate::constants::TABLE_ROLES;
use crate::constants::TABLE_ROLES_ALIASES;
use crate::get_database_conn;

fn filter_optional_array<T>(arr: &Option<Vec<T>>) -> bool {
    match arr {
        Some(arr) => arr.is_empty(),
        None => true,
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleProperties {
    #[serde(default)]
    #[serde(with = "humantime_serde", skip_serializing_if = "Option::is_none")]
    pub time_requirement: Option<Duration>,
    #[serde(skip_serializing_if = "filter_optional_array")]
    pub incompatible_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "filter_optional_array")]
    pub required_roles: Option<Vec<String>>,
    #[serde(skip_serializing)]
    pub aliases: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct CustomRole {
    pub id: RoleId,
    pub name: String,
    pub properties: RoleProperties,
    pub colour: Colour,
}

impl std::fmt::Debug for CustomRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} {:?})", self.name, self.id)
    }
}

pub async fn get_role(ctx: &Context, server_id: GuildId, role_name: &str) -> Option<CustomRole> {
    let mut conn = get_database_conn!(ctx);

    let (id, name, properties, colour): (u64, String, String, u32) = conn.exec_first(
        format!(
            "SELECT roles.role_id, role_name, role_properties, role_colour FROM {} AS roles
            JOIN {} AS aliases ON roles.role_id = aliases.role_id AND roles.server_id = aliases.server_id
            WHERE alias_name = :role_name AND roles.server_id = :server_id",
            TABLE_ROLES,
            TABLE_ROLES_ALIASES
        ),
        params! {
        "role_name" => role_name,
        "server_id" => server_id.0,
    }).await.ok()??;

    serde_json::from_str(&properties)
        .ok()
        .map(|properties| CustomRole {
            id: RoleId(id),
            name,
            properties,
            colour: Colour(colour),
        })
}

pub async fn add_role(ctx: &Context, server_id: GuildId, role: &CustomRole) -> CommandResult {
    let mut conn = get_database_conn!(ctx);
    let empty = Vec::new();

    let aliases = if let Some(aliases) = &role.properties.aliases {
        aliases
    } else {
        &empty
    };
    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND role_id = :role_id",
            TABLE_ROLES_ALIASES
        ),
        params! {
            "server_id" => server_id.0,
            "role_id" => role.id.0
        },
    )
    .await?;
    conn.exec_batch(
            format!(
                "INSERT INTO {} (server_id, alias_name, role_id) VALUES (:server_id, :alias_name, :role_id)",
                TABLE_ROLES_ALIASES,
            ),
            iter::once(&role.name).chain(aliases.iter()).map(|alias| params! {
                "server_id" => server_id.0,
                "alias_name" => format_role_name(alias),
                "role_id" => role.id.0,
            }),
        )
        .await?;

    let properties = serde_json::to_string(&role.properties)?;

    conn.exec_drop(format!(
        "REPLACE INTO {} (server_id, role_id, role_name, role_properties, role_colour) VALUES (:server_id, :role_id, :role_name, :role_properties, :colour)",
        TABLE_ROLES
    ), params! {
        "server_id" => server_id.0,
        "role_id" => role.id.0,
        "role_name" => &role.name,
        "role_properties" => properties,
        "colour" => role.colour.0
    }).await?;

    Ok(())
}

pub async fn delete_role(ctx: &Context, server_id: GuildId, role_id: RoleId) -> CommandResult {
    let mut conn = get_database_conn!(ctx);
    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND role_id = :role_id",
            TABLE_ROLES_ALIASES
        ),
        params! {
            "server_id" => server_id.0,
            "role_id" => role_id.0
        },
    )
    .await?;

    conn.exec_drop(
        format!(
            "DELETE FROM {} WHERE server_id = :server_id AND role_id = :role_id LIMIT 1",
            TABLE_ROLES
        ),
        params! {
            "server_id" => server_id.0,
            "role_id" => role_id.0
        },
    )
    .await?;

    Ok(())
}

pub async fn get_role_list(ctx: &Context, server_id: GuildId) -> Option<Vec<Vec<String>>> {
    let mut conn = get_database_conn!(ctx);

    let roles: Vec<u64> = conn
        .exec(
            format!(
                "SELECT role_id FROM {} WHERE server_id = :server_id",
                TABLE_ROLES
            ),
            params! {"server_id" => server_id.0},
        )
        .await
        .ok()?;

    let mut result = Vec::new();

    for id in roles {
        result.push(
            conn.exec(
                format!(
                    "SELECT alias_name FROM {} WHERE server_id = :server_id AND role_id = :role_id",
                    TABLE_ROLES_ALIASES
                ),
                params! {"server_id" => server_id.0, "role_id" => id},
            )
            .await
            .ok()?,
        );
    }

    Some(result)
}

pub async fn get_aliases(
    ctx: &Context,
    server_id: GuildId,
    role_id: RoleId,
) -> Option<Vec<String>> {
    let mut conn = get_database_conn!(ctx);

    conn.exec(
        format!(
            "SELECT alias_name FROM {} WHERE server_id = :server_id AND role_id = :role_id",
            TABLE_ROLES_ALIASES
        ),
        params! {
            "server_id" => server_id.0,
            "role_id" => role_id.0
        },
    )
    .await
    .ok()
}

#[cfg(test)]
mod tests {

    use super::RoleProperties;
    use std::time::Duration;

    #[test]
    fn test_role_properties() {
        let test_string = r#"
        { "time_requirement": "7d", "aliases": [] }
        "#;

        let test: RoleProperties = serde_json::from_str(test_string).unwrap();

        assert_eq!(
            RoleProperties {
                time_requirement: Some(Duration::new(604800, 0)),
                incompatible_roles: None,
                required_roles: None,
                aliases: Some(vec![])
            },
            test
        );

        let reverse = serde_json::to_string(&test).unwrap();

        assert_eq!(reverse, r#"{"time_requirement":"7days"}"#);

        let test_string = r#"
        { "aliases": ["test1", "test2"] }
        "#;

        let test: RoleProperties = serde_json::from_str(test_string).unwrap();

        assert_eq!(
            RoleProperties {
                time_requirement: None,
                incompatible_roles: None,
                required_roles: None,
                aliases: Some(vec![String::from("test1"), String::from("test2")])
            },
            test
        );
    }
}
