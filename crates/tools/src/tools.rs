use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::errors::ToolError;
use common::tools::config::ToolsSchemaConfig;
use common::tools::prelude::*;

use crate::Tool;
use crate::ToolSpec;
use crate::prelude::*;
use crate::project::PlanCreateTool;
use crate::project::PlanDeleteTool;
use crate::project::PlanGetTool;
use crate::project::PlanListTool;
use crate::project::PlanUpdateTool;
use crate::rg::Rg;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Tools {
  File(File),
  Memory(Memory),
  Host(Host),
  Project(Project),
  Skill(Skill),
  Rg(Rg),
}

#[async_trait]
impl Tool for Tools {
  fn tool_id(&self) -> ToolId {
    match self {
      Tools::File(File::FilesRead(_)) => ToolId::FilesRead,
      Tools::File(File::ApplyPatch(_)) => ToolId::ApplyPatch,
      Tools::Memory(Memory::Write(_)) => ToolId::MemoryWrite,
      Tools::Memory(Memory::Search(_)) => ToolId::MemorySearch,
      Tools::Host(Host::Shell(_)) => ToolId::Shell,
      Tools::Project(Project::GetPrimer(_)) => ToolId::PrimerGet,
      Tools::Project(Project::UpdatePrimer(_)) => ToolId::PrimerUpdate,
      Tools::Project(Project::PlanCreate(_)) => ToolId::PlanCreate,
      Tools::Project(Project::PlanList(_)) => ToolId::PlanList,
      Tools::Project(Project::PlanGet(_)) => ToolId::PlanGet,
      Tools::Project(Project::PlanUpdate(_)) => ToolId::PlanUpdate,
      Tools::Project(Project::PlanDelete(_)) => ToolId::PlanDelete,
      Tools::Skill(Skill::ListSkills(_)) => ToolId::ListSkills,
      Tools::Skill(Skill::GetReference(_)) => ToolId::GetReference,
      Tools::Skill(Skill::SkillScript(_)) => ToolId::SkillScript,
      Tools::Rg(Rg::Search(_)) => ToolId::Rg,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Tools::File(cmd) => cmd.run(context).await,
      Tools::Memory(cmd) => cmd.run(context).await,
      Tools::Host(cmd) => cmd.run(context).await,
      Tools::Project(cmd) => cmd.run(context).await,
      Tools::Skill(cmd) => cmd.run(context).await,
      Tools::Rg(cmd) => cmd.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(File::schema(config));
    schema.extend(Memory::schema(config));
    schema.extend(Host::schema(config));
    schema.extend(Project::schema(config));
    schema.extend(Skill::schema(config));
    schema.extend(Rg::schema(config));
    schema.extend(AskQuestionArgs::schema(config));
    schema.extend(TerminalArgs::schema(config));
    schema.extend(SubAgentArgs::schema(config));

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
      ToolId::PrimerGet => Ok(Tools::Project(Project::GetPrimer(GetPrimerTool))),
      ToolId::PrimerUpdate => {
        let args = serde_json::from_str::<UpdatePrimerArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::UpdatePrimer(UpdatePrimerTool { args })))
      }
      ToolId::PlanCreate => {
        let args = serde_json::from_str::<PlanCreateArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::PlanCreate(PlanCreateTool { args })))
      }
      ToolId::PlanList => {
        let args = serde_json::from_str::<PlanListArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::PlanList(PlanListTool { args })))
      }
      ToolId::PlanGet => {
        let args = serde_json::from_str::<PlanGetArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::PlanGet(PlanGetTool { args })))
      }
      ToolId::PlanUpdate => {
        let args = serde_json::from_str::<PlanUpdateArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::PlanUpdate(PlanUpdateTool { args })))
      }
      ToolId::PlanDelete => {
        let args = serde_json::from_str::<PlanDeleteArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Project(Project::PlanDelete(PlanDeleteTool { args })))
      }
      ToolId::MemoryWrite => {
        let args = serde_json::from_str::<MemoryWriteArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Memory(Memory::Write(WriteMemoryTool { args })))
      }
      ToolId::MemorySearch => {
        let args = serde_json::from_str::<MemorySearchArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Memory(Memory::Search(SearchMemoryTool { args })))
      }
      ToolId::ListSkills => {
        let args = serde_json::from_str::<ListSkillsArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Skill(Skill::ListSkills(ListSkillsTool { args })))
      }
      ToolId::GetReference => {
        let args = serde_json::from_str::<GetReferenceArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Skill(Skill::GetReference(GetReferenceTool { args })))
      }
      ToolId::SkillScript => {
        let args = serde_json::from_str::<SkillScriptArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Skill(Skill::SkillScript(SkillScriptTool { args })))
      }
      ToolId::ApplySkill => Err(ToolError::UnknownTool(tool_id.to_string()).into()),
      ToolId::Rg => {
        let args = serde_json::from_str::<RgSearchArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Rg(Rg::Search(RgSearchTool { args })))
      }
      ToolId::Unknown(name) | ToolId::Mcp(name) => Err(ToolError::UnknownTool(name.clone()).into()),
      // These tools are handled in the engine
      ToolId::AskQuestion | ToolId::SubAgent | ToolId::WebSearch | ToolId::Terminal => unreachable!(),
    }
  }
}

#[cfg(test)]
mod tests {
  use common::agent::AgentKind;

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
