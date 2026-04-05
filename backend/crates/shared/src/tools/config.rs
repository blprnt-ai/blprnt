use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct LlmModel {
  pub name:               String,
  pub slug:               String,
  pub context_length:     i64,
  pub supports_reasoning: bool,
  pub provider_slug:      Option<String>,
  pub enabled:            bool,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolRuntimeConfig {
  pub agent_home:   Option<PathBuf>,
  pub project_home: Option<PathBuf>,
  pub employee_id:  Option<String>,
  pub project_id:   Option<String>,
  pub run_id:       Option<String>,
  pub api_url:      Option<String>,
}

impl ToolRuntimeConfig {
  pub fn env_overrides(&self) -> HashMap<String, String> {
    let mut env = HashMap::new();

    if let Some(agent_home) = &self.agent_home {
      env.insert("AGENT_HOME".to_string(), agent_home.to_string_lossy().to_string());
    }

    if let Some(project_home) = &self.project_home {
      env.insert("PROJECT_HOME".to_string(), project_home.to_string_lossy().to_string());
    }

    if let Some(employee_id) = &self.employee_id {
      env.insert("BLPRNT_EMPLOYEE_ID".to_string(), employee_id.clone());
    }

    if let Some(project_id) = &self.project_id {
      env.insert("BLPRNT_PROJECT_ID".to_string(), project_id.clone());
    }

    if let Some(run_id) = &self.run_id {
      env.insert("BLPRNT_RUN_ID".to_string(), run_id.clone());
    }

    if let Some(api_url) = &self.api_url {
      env.insert("BLPRNT_API_URL".to_string(), api_url.clone());
    }

    env
  }
}
