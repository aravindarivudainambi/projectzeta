use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an end user who authors or delegates agent actions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}

/// Represents an isolated customer tenant.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
}

/// Represents a coarse-grained permission label.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

/// Returns a default permission used by skeleton handlers and tests.
pub fn placeholder_permission(resource: &str, action: &str) -> Permission {
    Permission {
        resource: resource.to_string(),
        action: action.to_string(),
    }
}
