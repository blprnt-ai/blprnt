use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolAllowList;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::RgSearchArgs;
use shared::tools::RgSearchPayload;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::config::ToolsSchemaConfig;
use tokio::process::Command;

use crate::Tool;
use crate::tool_use::ToolUseContext;
use crate::utils::get_workspace_root;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RgSearchTool {
  pub args: RgSearchArgs,
}

const MAX_RG_OUTPUT_TOKENS: usize = 500;
fn truncate_with_notice(output: &str, max_tokens: usize) -> String {
  let total_tokens = output.len() / 8;

  if total_tokens > max_tokens {
    let truncated = output.chars().skip(total_tokens - max_tokens * 8).collect::<String>();
    format!("{truncated}\n\n[output truncated to {max_tokens} tokens from {total_tokens}]")
  } else {
    output.to_string()
  }
}

#[async_trait]
impl Tool for RgSearchTool {
  fn tool_id(&self) -> ToolId {
    ToolId::Rg
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    run_rg(&context, &self.args).await
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::Rg, config.agent_kind) {
      return vec![];
    }

    let schema = schemars::schema_for!(RgSearchArgs);
    let json = serde_json::to_value(&schema).expect("[RgSearchArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[RgSearchArgs] properties is required"),
      "required": json.get("required").expect("[RgSearchArgs] required is required")
    });

    let name = schema.get("title").expect("[RgSearchArgs] title is required").clone();
    let description = schema.get("description").expect("[RgSearchArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

async fn run_rg(context: &ToolUseContext, args: &RgSearchArgs) -> Result<ToolUseResponse> {
  let mut cmd = Command::new("rg");

  let workspace_root = get_workspace_root(&context.working_directories, args.workspace_index);

  std::env::set_current_dir(&workspace_root).map_err(|e| ToolError::SpawnFailed(e.to_string()))?;

  let env_vars = crate::host::env::get_env();
  let mut cmd = cmd.envs(env_vars);

  if !args.flags.is_empty() {
    cmd = cmd.args(args.flags.iter().map(|s| s.as_str()));
  }

  cmd = cmd.arg(&args.pattern);
  cmd = cmd.arg(args.path.clone().unwrap_or(".".to_string()));

  let output = cmd.output().await.map_err(|e| ToolError::SpawnFailed(e.to_string()))?;
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stdout = truncate_with_notice(&stdout, MAX_RG_OUTPUT_TOKENS);

  if output.status.success() {
    let payload = RgSearchPayload { stdout }.into();
    return Ok(ToolUseResponseData::success(payload));
  }

  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  let stderr = truncate_with_notice(&stderr, MAX_RG_OUTPUT_TOKENS);
  let error = if !stderr.is_empty() { stderr } else { stdout };

  if error.trim().is_empty() {
    Ok(ToolUseResponseData::success(RgSearchPayload { stdout: "".to_string() }.into()))
  } else {
    Ok(ToolUseResponseData::success(RgSearchPayload { stdout: error }.into()))
  }
}
