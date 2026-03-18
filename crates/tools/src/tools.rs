use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::config::ToolsSchemaConfig;
use shared::tools::prelude::*;

use crate::Tool;
use crate::ToolSpec;
use crate::prelude::*;
use crate::rg::Rg;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Tools {
  File(File),
  Host(Host),
  Skill(Skill),
  Rg(Rg),
}

#[async_trait]
impl Tool for Tools {
  fn tool_id(&self) -> ToolId {
    match self {
      Tools::File(File::FilesRead(_)) => ToolId::FilesRead,
      Tools::File(File::ApplyPatch(_)) => ToolId::ApplyPatch,
      Tools::Host(Host::Shell(_)) => ToolId::Shell,
      Tools::Skill(Skill::SkillScript(_)) => ToolId::SkillScript,
      Tools::Rg(Rg::Search(_)) => ToolId::Rg,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Tools::File(cmd) => cmd.run(context).await,
      Tools::Host(cmd) => cmd.run(context).await,
      Tools::Skill(cmd) => cmd.run(context).await,
      Tools::Rg(cmd) => cmd.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(File::schema(config));
    schema.extend(Host::schema(config));
    schema.extend(Skill::schema(config));
    schema.extend(Rg::schema(config));
    schema.extend(TerminalArgs::schema(config));

    schema
  }
}

impl TryFrom<(&ToolId, &str)> for Tools {
  type Error = anyhow::Error;

  fn try_from((tool_id, args): (&ToolId, &str)) -> Result<Self> {
    match tool_id {
      ToolId::FilesRead => {
        let args = serde_json::from_str::<FilesReadArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::File(File::FilesRead(FilesReadTool { args })))
      }
      ToolId::ApplyPatch => {
        let args = serde_json::from_str::<ApplyPatchArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::File(File::ApplyPatch(ApplyPatchTool { args })))
      }
      ToolId::Shell => {
        let args = serde_json::from_str::<ShellArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Host(Host::Shell(ShellTool { args })))
      }
      ToolId::SkillScript => {
        let args = serde_json::from_str::<SkillScriptArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Skill(Skill::SkillScript(SkillScriptTool { args })))
      }
      ToolId::Rg => {
        let args = serde_json::from_str::<RgSearchArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Rg(Rg::Search(RgSearchTool { args })))
      }
      _ => Err(ToolError::UnknownTool(tool_id.to_string()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use shared::agent::AgentKind;

  use super::*;

  #[test]
  fn test_schema_with_providers() {
    let agent_kind = AgentKind::Planner;
    #[allow(unused_variables)]
    let working_directories = WorkingDirectories::new(vec![]);
    let schema = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           agent_kind,
      working_directories:  working_directories,
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });
    println!("{}", serde_json::to_string_pretty(&schema).unwrap_or_default());
  }
}
