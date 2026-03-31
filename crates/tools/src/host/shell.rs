use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolId;
use shared::tools::ShellPayload;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::host::ShellArgs;

use crate::Tool;
use crate::ToolSpec;
use crate::host::child::Child;
use crate::tool_use::ToolUseContext;
use crate::utils::get_workspace_root;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShellTool {
  pub args: ShellArgs,
}

#[derive(Clone, Debug)]
pub struct ShellConfig {
  pub timeout:          Duration,
  pub buffer_size:      usize,
  pub max_output_lines: usize,
}

impl Default for ShellConfig {
  fn default() -> Self {
    Self { timeout: Duration::from_secs(120), buffer_size: 8192, max_output_lines: 10000 }
  }
}

#[derive(Clone, Debug)]
pub enum ProcOutput {
  Stdout(String),
  Stderr(String),
  ExitCode(i32),
  Event { event_kind: String, event_data: serde_json::Value },
}

#[async_trait]
impl Tool for ShellTool {
  fn tool_id(&self) -> ToolId {
    ToolId::Shell
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let workspace_root = get_workspace_root(&context.working_directories, self.args.workspace_index);

    #[cfg(not(target_os = "windows"))]
    let (stdout, stderr, exit_code) = Child::spawn(
      &workspace_root,
      self.args.command.clone(),
      self.args.args.clone(),
      self.args.timeout,
      context.runtime_config.clone(),
      context.sandbox.clone(),
    )
    .await?;
    #[cfg(target_os = "windows")]
    let (stdout, stderr, exit_code) = Child::spawn(
      &workspace_root,
      self.args.command.clone(),
      self.args.args.clone(),
      self.args.timeout,
      context.runtime_config.clone(),
      context.sandbox.clone(),
    )
    .await?;

    let stdout = String::from_utf8(stdout).unwrap();
    let stdout = truncate_output(&stdout, 500);
    let stderr = String::from_utf8(stderr).unwrap();
    let stderr = truncate_output(&stderr, 500);
    let exit_code = exit_code.code().unwrap_or(0);

    let payload = ShellPayload { stdout, stderr, exit_code }.into();

    Ok(ToolUseResponseData::success(payload))
  }

  fn schema() -> Vec<ToolSpec> {
    let schema = schemars::schema_for!(ShellArgs);
    let json = serde_json::to_value(&schema).expect("[ShellArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[ShellArgs] properties is required"),
      "required": json.get("required").expect("[ShellArgs] required is required")
    });

    let name = schema.get("title").expect("[ShellArgs] title is required").clone();
    let description = schema.get("description").expect("[ShellArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

fn truncate_output(output: &str, max_tokens: usize) -> String {
  let total_tokens = output.chars().count() / 8;

  if total_tokens > max_tokens {
    let skip_amount = total_tokens - max_tokens;
    let truncated = output.chars().skip(skip_amount * 8).collect::<String>();
    format!("{truncated}\n\n[output truncated to {max_tokens} tokens from {total_tokens}]")
  } else {
    output.to_string()
  }
}
