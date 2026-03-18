use crate::agent::prelude::*;

pub struct ToolAllowList;

impl ToolAllowList {
  pub fn is_tool_allowed_and_enabled(tool_id: ToolId, agent_kind: AgentKind) -> bool {
    ToolAllowList::is_tool_allowed_and_enabled_for_runtime(tool_id, agent_kind)
  }

  pub fn is_tool_allowed_and_enabled_for_runtime(tool_id: ToolId, agent_kind: AgentKind) -> bool {
    if !ToolAllowList::is_tool_enabled(&tool_id) {
      return false;
    }

    if let ToolId::Mcp(_) = &tool_id {
      return true;
    }

    if let ToolId::Unknown(name) = &tool_id
      && name.starts_with("mcp__")
    {
      return true;
    }

    match agent_kind {
      AgentKind::Crew => true,

      AgentKind::Researcher => {
        matches!(tool_id, ToolId::FilesRead | ToolId::Rg)
      }

      AgentKind::Planner => matches!(tool_id, |ToolId::FilesRead| ToolId::Rg),

      AgentKind::Executor => matches!(
        tool_id,
        ToolId::FilesRead | ToolId::ApplyPatch | ToolId::Shell | ToolId::Terminal | ToolId::Rg | ToolId::SkillScript
      ),

      AgentKind::Verifier => {
        matches!(tool_id, ToolId::FilesRead | ToolId::Rg | ToolId::Shell | ToolId::Terminal | ToolId::SkillScript)
      }

      AgentKind::Designer => {
        matches!(tool_id, ToolId::FilesRead | ToolId::Rg)
      }
    }
  }

  fn is_tool_enabled(tool_id: &ToolId) -> bool {
    match tool_id {
      // File tools
      ToolId::FilesRead => true,
      ToolId::ApplyPatch => true,

      // Shell tools
      ToolId::Shell => true,
      ToolId::Terminal => true,

      // Codebase tools
      ToolId::Rg => true,

      ToolId::SkillScript => true,

      // Unknown tools
      ToolId::Mcp(name) => name.starts_with("mcp__"),
      ToolId::Unknown(name) => name.starts_with("mcp__"),
    }
  }
}
