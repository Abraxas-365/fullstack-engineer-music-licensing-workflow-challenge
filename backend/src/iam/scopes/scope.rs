use std::collections::HashMap;
use std::sync::LazyLock;

// Super scope — full access to everything
pub const SCOPE_ALL: &str = "*";

// User management scopes
pub const SCOPE_USERS_ALL: &str = "users:*";
pub const SCOPE_USERS_READ: &str = "users:read";
pub const SCOPE_USERS_WRITE: &str = "users:write";
pub const SCOPE_USERS_DELETE: &str = "users:delete";

// Role management scopes
pub const SCOPE_ROLES_ALL: &str = "roles:*";
pub const SCOPE_ROLES_READ: &str = "roles:read";
pub const SCOPE_ROLES_WRITE: &str = "roles:write";
pub const SCOPE_ROLES_DELETE: &str = "roles:delete";
pub const SCOPE_ROLES_ASSIGN: &str = "roles:assign";

// Scope management scopes
pub const SCOPE_SCOPES_ALL: &str = "scopes:*";
pub const SCOPE_SCOPES_READ: &str = "scopes:read";
pub const SCOPE_SCOPES_WRITE: &str = "scopes:write";
pub const SCOPE_SCOPES_ASSIGN: &str = "scopes:assign";

// ============================================================================
// Catalog
// ============================================================================

pub static SCOPE_CATEGORIES: LazyLock<HashMap<&str, Vec<&str>>> = LazyLock::new(|| {
    HashMap::from([
        (
            "Users",
            vec![
                SCOPE_USERS_ALL,
                SCOPE_USERS_READ,
                SCOPE_USERS_WRITE,
                SCOPE_USERS_DELETE,
            ],
        ),
        (
            "Roles",
            vec![
                SCOPE_ROLES_ALL,
                SCOPE_ROLES_READ,
                SCOPE_ROLES_WRITE,
                SCOPE_ROLES_DELETE,
                SCOPE_ROLES_ASSIGN,
            ],
        ),
        (
            "Scopes",
            vec![
                SCOPE_SCOPES_ALL,
                SCOPE_SCOPES_READ,
                SCOPE_SCOPES_WRITE,
                SCOPE_SCOPES_ASSIGN,
            ],
        ),
    ])
});

pub static SCOPE_DESCRIPTIONS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        (SCOPE_ALL, "Full access to all system resources"),
        // Users
        (SCOPE_USERS_ALL, "Full access to user management"),
        (SCOPE_USERS_READ, "View users"),
        (SCOPE_USERS_WRITE, "Create and edit users"),
        (SCOPE_USERS_DELETE, "Delete users"),
        // Roles
        (SCOPE_ROLES_ALL, "Full access to role management"),
        (SCOPE_ROLES_READ, "View roles"),
        (SCOPE_ROLES_WRITE, "Create and edit roles"),
        (SCOPE_ROLES_DELETE, "Delete roles"),
        (SCOPE_ROLES_ASSIGN, "Assign roles to users"),
        // Scopes
        (SCOPE_SCOPES_ALL, "Full access to scope management"),
        (SCOPE_SCOPES_READ, "View available scopes"),
        (SCOPE_SCOPES_WRITE, "Set and modify scopes"),
        (SCOPE_SCOPES_ASSIGN, "Add or remove scopes"),
    ])
});

// ============================================================================
// Helpers
// ============================================================================

/// Returns the description for a given scope.
pub fn get_scope_description(scope: &str) -> &str {
    SCOPE_DESCRIPTIONS
        .get(scope)
        .copied()
        .unwrap_or("No description available")
}

/// Returns all defined scopes (flattened from categories).
pub fn get_all_scopes() -> Vec<&'static str> {
    SCOPE_CATEGORIES
        .values()
        .flat_map(|s| s.iter().copied())
        .collect()
}

/// Checks if a scope string is valid (exists in the catalog or is the super scope).
pub fn validate_scope(scope: &str) -> bool {
    if scope == SCOPE_ALL {
        return true;
    }
    SCOPE_CATEGORIES.values().any(|s| s.contains(&scope))
}

/// Returns the category name for a scope.
pub fn get_scope_category(scope: &str) -> &str {
    for (category, scopes) in SCOPE_CATEGORIES.iter() {
        if scopes.contains(&scope) {
            return category;
        }
    }
    "Unknown"
}

/// Expands a wildcard scope to all matching concrete scopes.
/// e.g. "users:*" -> ["users:read", "users:write", "users:delete"]
pub fn expand_wildcard_scope(wildcard: &str) -> Vec<&'static str> {
    if wildcard == SCOPE_ALL {
        return get_all_scopes();
    }

    let prefix = match wildcard.strip_suffix(":*") {
        Some(p) => p,
        None => return vec![wildcard_to_static(wildcard)],
    };

    let prefix_colon = format!("{prefix}:");
    SCOPE_CATEGORIES
        .values()
        .flat_map(|s| s.iter().copied())
        .filter(|s| s.starts_with(&prefix_colon))
        .collect()
}

/// Checks if a set of scopes contains a specific scope,
/// accounting for wildcards. e.g. ["users:*"] contains "users:read".
pub fn scopes_contain(scopes: &[String], target: &str) -> bool {
    for scope in scopes {
        if scope == SCOPE_ALL || scope == target {
            return true;
        }
        if let Some(prefix) = scope.strip_suffix(":*") {
            if target.starts_with(&format!("{prefix}:")) {
                return true;
            }
        }
    }
    false
}

fn wildcard_to_static(s: &str) -> &'static str {
    // For non-wildcard scopes, try to match against known constants
    for scopes in SCOPE_CATEGORIES.values() {
        for &scope in scopes {
            if scope == s {
                return scope;
            }
        }
    }
    // Leak the string — this only happens for unknown scopes
    Box::leak(s.to_string().into_boxed_str())
}
