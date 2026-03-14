use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::blprnt::Blprnt;
use common::errors::ToolError;
use common::tokenizer::Tokenizer;
use common::tools::RgSearchArgs;
use common::tools::RgSearchPayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;
use tauri_plugin_shell::ShellExt;

use crate::Tool;
use crate::tool_use::ToolUseContext;
use crate::utils::get_workspace_root;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RgSearchTool {
  pub args: RgSearchArgs,
}

const MAX_RG_OUTPUT_TOKENS: usize = 500;
fn truncate_with_notice(output: &str, max_tokens: usize) -> String {
  let total_tokens = Tokenizer::count_string_tokens(output) as usize;

  if total_tokens > max_tokens {
    let truncated = Tokenizer::truncate_output(output, max_tokens);
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
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::Rg, config.agent_kind, config.is_subagent) {
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
  let cmd = Blprnt::handle().shell().sidecar("rg").map_err(|e| ToolError::SpawnFailed(e.to_string()))?;

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

#[cfg(all(test, unix))]
mod tests {
  use std::os::unix::fs::PermissionsExt;
  use std::path::PathBuf;

  use common::agent::AgentKind;
  use common::sandbox_flags::SandboxFlags;
  use common::tools::ToolUseResponseSuccess;
  use persistence::prelude::SurrealId;
  use tauri::test::mock_builder;
  use tauri::test::mock_context;
  use tauri::test::noop_assets;

  use super::*;

  #[tokio::test]
  async fn test_rg() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let tauri_src =
      PathBuf::from(&manifest_dir).ancestors().find(|p| p.join("tauri-src").exists()).unwrap().join("tauri-src");
    std::env::set_current_dir(&tauri_src).unwrap();

    let binaries_dir = tauri_src.join("binaries");
    println!("Binaries dir: {:?}", binaries_dir);
    println!("Binaries dir exists: {}", binaries_dir.exists());

    if binaries_dir.exists() {
      for entry in std::fs::read_dir(&binaries_dir).unwrap() {
        println!("  Found: {:?}", entry.unwrap().path());
      }
    }

    // Check for the exact file Tauri expects
    let expected = binaries_dir.join("rg-aarch64-apple-darwin");
    println!("Expected binary: {:?}", expected);
    println!("Expected exists: {}", expected.exists());

    let metadata = std::fs::metadata(&expected).unwrap();
    println!("Is executable: {}", metadata.permissions().mode() & 0o111 != 0);

    let direct_result = std::process::Command::new(&expected).arg("--version").output();
    println!("Direct execution: {:?}", direct_result);

    let mut context = mock_context(noop_assets());
    let config = context.config_mut();
    config.bundle.external_bin = Some(vec!["binaries/rg".to_string()]);

    let app = mock_builder().plugin(tauri_plugin_shell::init()).build(context).unwrap();
    let test_dir = PathBuf::from("/private/tmp/rg_test");

    std::fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("fake.txt");
    std::fs::write(test_file, "Hello world!").unwrap();

    sandbox::sandbox_test_setup(&test_dir).await.unwrap();
    let working_directories = vec![test_dir.clone()];

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      working_directories,
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );

    println!("Current dir: {:?}", std::env::current_dir());
    println!("Target: {}", std::env::consts::ARCH);

    match app.shell().sidecar("rg") {
      Ok(_) => println!("Sidecar resolved successfully"),
      Err(e) => println!("Sidecar error: {:?}", e),
    }

    let payload = run_rg(
      &context,
      &RgSearchArgs {
        pattern:         "fake".to_string(),
        path:            Some(test_dir.to_string_lossy().to_string()),
        flags:           vec![],
        workspace_index: None,
      },
    )
    .await;

    match payload {
      Ok(ToolUseResponse::Success(ToolUseResponseSuccess { data: ToolUseResponseData::RgSearch(payload), .. })) => {
        print!("{}", payload.stdout);
      }
      Ok(ToolUseResponse::Error(e)) => println!("ToolUseError RgSearchPayload: {}", e.error),
      Err(e) => println!("ToolError RgSearchPayload: {}", e),
      _ => println!("expected RgSearchPayload"),
    }
  }
}
