use std::collections::HashMap;
use std::path::PathBuf;

use common::agent::AgentKind;
use common::before_all;
use common::shared::prelude::PromptParams;
use prompt::render_prompt;
use tracing_subscriber::EnvFilter;

async fn init_tracing() {
  let filter = EnvFilter::new("info");
  tracing_subscriber::fmt().with_env_filter(filter).init();
}

before_all!(init = init_tracing);

#[tokio::test]
async fn test_prompt_templates() {
  let workspace_roots = vec![PathBuf::from("/tmp/blprnt-test")];
  let params = PromptParams {
    agent_kind:      AgentKind::Crew,
    personality:     "test personality".to_string(),
    workspace_roots: workspace_roots.clone(),
    primer:          None,
    current_skills:  vec![],
    plan_context:    None,
    mcp_details:     HashMap::new(),
    memory:          String::new(),
  };

  let template = render_prompt(params);

  tracing::info!("Template: {}", template);
}
