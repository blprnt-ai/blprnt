use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::tokenizer::Tokenizer;
use common::tools::ShellPayload;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;
use common::tools::host::ShellArgs;

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
      context.sandbox_flags,
    )
    .await?;
    #[cfg(target_os = "windows")]
    let (stdout, stderr, exit_code) =
      Child::spawn(&workspace_root, self.args.command.clone(), self.args.args.clone(), self.args.timeout).await?;

    let stdout = String::from_utf8(stdout).unwrap();
    let stdout = Tokenizer::truncate_output(&stdout, 500);
    let stderr = String::from_utf8(stderr).unwrap();
    let stderr = Tokenizer::truncate_output(&stderr, 500);
    let exit_code = exit_code.code().unwrap_or(0);

    let payload = ShellPayload { stdout, stderr, exit_code }.into();

    Ok(ToolUseResponseData::success(payload))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::Shell, config.agent_kind, config.is_subagent) {
      return vec![];
    }

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

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use common::agent::AgentKind;
  use common::sandbox_flags::SandboxFlags;
  use persistence::prelude::SurrealId;

  use super::*;

  #[tokio::test]
  async fn test_shell_tool() {
    let tool = ShellTool {
      args: ShellArgs {
        command:         "echo".to_string(),
        args:            vec!["Hello, world!".to_string()],
        timeout:         Some(10),
        workspace_index: None,
      },
    };
    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from("/tmp")],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );
    let result = tool.run(context).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_npx_create_next_app() {
    let tool = ShellTool {
      args: ShellArgs {
        command:         "npx".to_string(),
        args:            vec![
          "create-next-app@latest".to_string(),
          ".".to_string(),
          "--ts".to_string(),
          "--eslint".to_string(),
          "--app".to_string(),
          "--use-npm".to_string(),
          "--no-tailwind".to_string(),
          "--src-dir".to_string(),
          "--import-alias".to_string(),
          "@/*".to_string(),
          "--yes".to_string(),
        ],
        timeout:         Some(120000),
        workspace_index: None,
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &PathBuf::from("/Users/supagoku/projects/new-peppa")).await.unwrap();

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from("/Users/supagoku/projects/new-peppa")],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );
    let result = tool.run(context).await;

    println!("Result: {:?}", result);
  }

  #[tokio::test]
  async fn fun_pythong() {
    // ""

    let tool = ShellTool {
      args: ShellArgs {
        command: "python".to_string(),
        args:    vec![
          "-c".to_string(),
          "import random, pathlib, datetime, subprocess, sys\nn=random.randint(1000,9999)\nfn=pathlib.Path(f'random_script_{n}.py')\nfn.write_text(\"import random, datetime\\nprint('Random value:', random.randint(1, 1000000))\\nprint('Timestamp:', datetime.datetime.now().isoformat())\\n\", encoding='utf-8')\nprint(fn.name)\nsubprocess.run([sys.executable, str(fn)], check=False)\n".to_string(),
        ],
        timeout: Some(120000),
        workspace_index: None,
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &PathBuf::from("/Users/supagoku/projects/new-peppa")).await.unwrap();

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from("/Users/supagoku/projects/new-peppa")],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );
    let result = tool.run(context).await;

    println!("Result: {:?}", result);
  }

  #[tokio::test]
  async fn get_child_item() {
    // ""

    let tool = ShellTool {
      args: ShellArgs {
        command:         "Get-ChildItem".to_string(),
        args:            vec!["-Path".to_string(), "Columbus".to_string()],
        timeout:         Some(120000),
        workspace_index: None,
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &PathBuf::from("C:\\Users\\Vitaliy\\projects\\Columbus")).await.unwrap();

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from("C:\\Users\\Vitaliy\\projects\\Columbus")],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );
    let result = tool.run(context).await;

    println!("Result: {:?}", result);
  }
}
