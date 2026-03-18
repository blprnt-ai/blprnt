use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolAllowList;
use shared::agent::ToolId;
use shared::skills_utils::SkillsUtils;
use shared::tools::ShellArgs;
use shared::tools::SkillScriptArgs;
use shared::tools::SkillScriptPayload;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::host::ShellTool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct SkillScriptTool {
  pub args: SkillScriptArgs,
}

#[async_trait]
impl Tool for SkillScriptTool {
  fn tool_id(&self) -> ToolId {
    ToolId::SkillScript
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let current_skills = context.current_skills.clone();

    if current_skills.is_empty() {
      return Err(anyhow::anyhow!("No current skill loaded."));
    }

    for skill_name in current_skills {
      if let Ok(script_path) = SkillsUtils::get_skill_script_path(&skill_name, &self.args.name) {
        let shell_args = build_shell_args(&script_path, self.args.args.clone());
        let response = ShellTool { args: shell_args }.run(context.clone()).await?;
        return map_shell_response(response);
      }
    }

    Err(anyhow::anyhow!("Script '{}' not found under any active skill.", self.args.name))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::SkillScript, config.agent_kind) {
      return vec![];
    }

    let schema = schemars::schema_for!(SkillScriptArgs);
    let json = serde_json::to_value(&schema).expect("[SkillScriptArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[SkillScriptArgs] properties is required"),
      "required": json.get("required").expect("[SkillScriptArgs] required is required")
    });

    let name = schema.get("title").expect("[SkillScriptArgs] title is required").clone();
    let description = schema.get("description").expect("[SkillScriptArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

fn build_shell_args(script_path: &Path, args: Vec<String>) -> ShellArgs {
  let script_path = script_path.to_string_lossy().to_string();
  let extension = script_path.rsplit('.').next().map(|ext| ext.to_ascii_lowercase());

  let (command, mut command_args) = match extension.as_deref() {
    Some("py") => ("python3".to_string(), vec![script_path]),
    Some("js") | Some("mjs") | Some("cjs") => ("node".to_string(), vec![script_path]),
    Some("sh") | Some("bash") => ("sh".to_string(), vec![script_path]),
    _ => (script_path, Vec::new()),
  };

  command_args.extend(args);

  ShellArgs { command, args: command_args, timeout: None, workspace_index: None }
}

fn map_shell_response(response: ToolUseResponse) -> Result<ToolUseResponse> {
  match response {
    ToolUseResponse::Success(success) => match success.data {
      ToolUseResponseData::Shell(payload) => {
        let error = if payload.exit_code != 0 {
          let mut errors = vec![format!("Script exited with code {}", payload.exit_code)];

          if !payload.stderr.trim().is_empty() {
            errors.push(payload.stderr);
          }

          Some(errors.join("\n\n"))
        } else {
          None
        };

        Ok(ToolUseResponseData::success(SkillScriptPayload { result: payload.stdout, error }.into()))
      }
      other => Err(anyhow::anyhow!("Unexpected shell response payload: {:?}", other)),
    },
    ToolUseResponse::Error(error) => {
      Ok(ToolUseResponseData::success(SkillScriptPayload { result: String::new(), error: Some(error.error) }.into()))
    }
  }
}
