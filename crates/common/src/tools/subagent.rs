use schemars::schema_for;
use serde_json::Value;
use surrealdb_types::SurrealValue;

use super::ToolSpec;
use super::ToolUseResponseData;
use crate::agent::prelude::*;
use crate::tools::config::ToolsSchemaConfig;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(title = "subagent", description = "Spawns a separate LLM session to handle a delegated subtask.")]
pub struct SubAgentArgs {
  pub prompt: String,

  #[schemars(description = "A 3-5 word description of the subagent's purpose, user facing.")]
  pub name: String,

  #[serde(default = "AgentKind::default")]
  pub agent_kind:     AgentKind,
  pub model_override: Option<String>,
  pub subagent_id:    Option<String>,
}

impl SubAgentArgs {
  pub fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::SubAgent, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schema_for!(SubAgentArgs);
    let mut json = serde_json::to_value(&schema).expect("[SubAgentArgs] schema is required");

    let mut properties = match json.get_mut("properties") {
      Some(properties) => properties.clone(),
      None => {
        tracing::error!("[SubAgentArgs] properties is required");
        return vec![];
      }
    };
    let Some(properties) = properties.as_object_mut() else {
      tracing::error!("[SubAgentArgs] properties must be an object");
      return vec![];
    };

    let Some(model_override) = properties.get_mut("model_override") else {
      tracing::error!("[SubAgentArgs] model_override is required");
      return vec![];
    };
    let Some(model_override) = model_override.as_object_mut() else {
      tracing::error!("[SubAgentArgs] model_override must be an object");
      return vec![];
    };

    let available_models = config.enabled_models.iter().map(|m| Value::String(m.slug.clone())).collect::<Vec<Value>>();

    let model_enum = Value::Array(available_models);
    model_override.insert("enum".to_string(), model_enum);

    let model_override_type =
      Value::Array(vec![Value::String("string".to_string()), Value::String("null".to_string())]);
    model_override.insert("type".to_string(), model_override_type);

    let model_override = Value::Object(model_override.clone());

    properties.insert("model_override".to_string(), model_override);

    let params = serde_json::json!({
      "type": "object",
      "properties": properties,
      "required": json.get("required").expect("[SubAgentArgs] required is required")
    });

    let name = schema.get("title").expect("[SubAgentArgs] title is required").clone();
    let description = schema.get("description").expect("[SubAgentArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SubAgentPayload {
  pub result:      String,
  pub subagent_id: Option<String>,
}

impl From<SubAgentPayload> for ToolUseResponseData {
  fn from(payload: SubAgentPayload) -> Self {
    Self::SubAgent(payload)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Artifact {
  pub name:          String,
  pub content:       String,
  pub artifact_type: ArtifactType,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ArtifactType {
  Text,
  File,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SubAgentMetadata {
  pub tokens_used: u32,
  pub duration_ms: u64,
  pub model_used:  String,
  pub parent_id:   String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tools::WorkingDirectories;

  #[test]
  fn test_subagent_schema() {
    let schema = SubAgentArgs::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Planner,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });
    let schema_json = serde_json::to_value(&schema).unwrap_or_default();
    println!("schema_json: {}", serde_json::to_string_pretty(&schema_json).unwrap_or_default());
  }
}
