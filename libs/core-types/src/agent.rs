use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes a persisted agent configuration contract.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentConfig {
    pub id: Uuid,
    pub name: String,
    pub trigger: Trigger,
    pub steps: Vec<AgentStep>,
}

/// Defines what initiates an agent run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Trigger {
    Manual,
    Schedule { cron: String },
    Event { source: String, event: String },
}

/// Captures one ordered unit of work in an agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentStep {
    pub id: Uuid,
    pub name: String,
}

/// Captures a versioned snapshot of agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentVersion {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub version: i32,
    pub snapshot_json: String,
}

/// Returns a scaffolded example agent config for tests, previews, or schema generation.
///
/// This helper exists so downstream crates have a stable, typed seed object while
/// business logic is still unimplemented.
pub fn sample_agent_config() -> AgentConfig {
    AgentConfig {
        id: Uuid::nil(),
        name: "Example Agent".to_string(),
        trigger: Trigger::Manual,
        steps: vec![AgentStep {
            id: Uuid::nil(),
            name: "Placeholder Step".to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::AgentConfig;
    use schemars::schema_for;
    use serde_json::Value;

    fn has_null_type_anywhere(value: &Value) -> bool {
        match value {
            Value::Object(map) => map.iter().any(|(key, nested)| {
                (key == "type"
                    && match nested {
                        Value::String(s) => s == "null",
                        Value::Array(values) => values
                            .iter()
                            .any(|v| matches!(v, Value::String(s) if s == "null")),
                        _ => false,
                    })
                    || has_null_type_anywhere(nested)
            }),
            Value::Array(values) => values.iter().any(has_null_type_anywhere),
            _ => false,
        }
    }

    #[test]
    fn agent_config_schema_has_expected_required_fields_and_no_implicit_nulls() {
        let schema = schema_for!(AgentConfig);
        let schema_value =
            serde_json::to_value(schema).expect("schema serialization should succeed");

        let required = &schema_value["required"];
        let required_has_expected_fields = match required.as_array() {
            Some(fields) => {
                fields.contains(&Value::String("id".to_string()))
                    && fields.contains(&Value::String("name".to_string()))
                    && fields.contains(&Value::String("trigger".to_string()))
                    && fields.contains(&Value::String("steps".to_string()))
            }
            None => false,
        };

        assert!(
            required_has_expected_fields,
            "AgentConfig schema must require id, name, trigger, and steps."
        );

        assert!(
            !has_null_type_anywhere(&schema_value),
            "AgentConfig schema should not contain null type fields unless intentionally optional."
        );
    }
}
