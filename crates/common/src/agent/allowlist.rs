use crate::agent::prelude::*;

pub struct ToolAllowList;

impl ToolAllowList {
  pub fn is_tool_allowed_and_enabled(tool_id: ToolId, agent_kind: AgentKind, is_subagent: bool) -> bool {
    ToolAllowList::is_tool_allowed_and_enabled_for_runtime(tool_id, agent_kind, is_subagent, true)
  }

  pub fn is_tool_allowed_and_enabled_for_runtime(
    tool_id: ToolId,
    agent_kind: AgentKind,
    is_subagent: bool,
    memory_tools_enabled: bool,
  ) -> bool {
    if !memory_tools_enabled && matches!(tool_id, ToolId::MemoryWrite | ToolId::MemorySearch) {
      return false;
    }

    if !ToolAllowList::is_tool_enabled(&tool_id) {
      return false;
    }

    if (tool_id == ToolId::AskQuestion || tool_id == ToolId::SubAgent) && is_subagent {
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

    if is_subagent {
      match tool_id {
        ToolId::PlanCreate | ToolId::PlanUpdate => return agent_kind == AgentKind::Planner,
        ToolId::PlanDelete => return false,
        _ => {}
      }
    }

    match agent_kind {
      AgentKind::Crew => true,

      AgentKind::Researcher => {
        matches!(
          tool_id,
          ToolId::FilesRead | ToolId::PrimerGet | ToolId::MemorySearch | ToolId::Rg | ToolId::GetReference
        )
      }

      AgentKind::Planner => matches!(
        tool_id,
        ToolId::PrimerGet
          | ToolId::PrimerUpdate
          | ToolId::PlanCreate
          | ToolId::PlanList
          | ToolId::PlanGet
          | ToolId::PlanUpdate
          | ToolId::PlanDelete
          | ToolId::FilesRead
          | ToolId::SubAgent
          | ToolId::MemoryWrite
          | ToolId::MemorySearch
          | ToolId::GetReference
          | ToolId::Rg
      ),

      AgentKind::Executor => matches!(
        tool_id,
        ToolId::FilesRead
          | ToolId::ApplyPatch
          | ToolId::Shell
          | ToolId::Terminal
          | ToolId::PrimerGet
          | ToolId::PrimerUpdate
          | ToolId::PlanCreate
          | ToolId::PlanList
          | ToolId::PlanGet
          | ToolId::PlanUpdate
          | ToolId::PlanDelete
          | ToolId::MemoryWrite
          | ToolId::MemorySearch
          | ToolId::Rg
          | ToolId::GetReference
          | ToolId::SkillScript
      ),

      AgentKind::Verifier => {
        matches!(
          tool_id,
          ToolId::FilesRead
            | ToolId::PrimerGet
            | ToolId::PlanList
            | ToolId::PlanGet
            | ToolId::MemoryWrite
            | ToolId::MemorySearch
            | ToolId::Rg
            | ToolId::Shell
            | ToolId::Terminal
            | ToolId::GetReference
            | ToolId::SkillScript
        )
      }

      AgentKind::Designer => {
        matches!(
          tool_id,
          ToolId::PrimerGet | ToolId::FilesRead | ToolId::MemorySearch | ToolId::Rg | ToolId::GetReference
        )
      }
    }
  }

  fn is_tool_enabled(tool_id: &ToolId) -> bool {
    match tool_id {
      // File tools
      ToolId::FilesRead => true,
      ToolId::ApplyPatch => true,

      // Dir tools

      // Shell tools
      ToolId::Shell => true,
      ToolId::Terminal => true,

      // Planning tools
      ToolId::PlanCreate => true,
      ToolId::PlanList => true,
      ToolId::PlanGet => true,
      ToolId::PlanUpdate => true,
      ToolId::PlanDelete => true,
      ToolId::AskQuestion => true,

      // Primer tools
      ToolId::PrimerGet => true,
      ToolId::PrimerUpdate => true,

      // Memory tools
      ToolId::MemoryWrite => true,
      ToolId::MemorySearch => true,

      // Codebase tools
      ToolId::Rg => true,

      // Subagent tools
      ToolId::SubAgent => true,

      // Web search is not a native tool, but we need the enum variant
      ToolId::WebSearch => false,

      // Get skill tool
      ToolId::ListSkills => false,
      ToolId::ApplySkill => false,
      ToolId::GetReference => true,
      ToolId::SkillScript => true,

      // Unknown tools
      ToolId::Mcp(name) => name.starts_with("mcp__"),
      ToolId::Unknown(name) => name.starts_with("mcp__"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ToolAllowList;
  use crate::agent::AgentKind;
  use crate::agent::ToolId;

  #[test]
  fn mcp_unknown_tools_are_allowed_for_main_agent() {
    let allowed = ToolAllowList::is_tool_allowed_and_enabled(
      ToolId::Mcp("mcp__server-1__search".to_string()),
      AgentKind::Planner,
      false,
    );

    assert!(allowed);
  }

  #[test]
  fn mcp_unknown_tools_are_allowed_for_subagent() {
    let allowed = ToolAllowList::is_tool_allowed_and_enabled(
      ToolId::Mcp("mcp__server-2__query".to_string()),
      AgentKind::Verifier,
      true,
    );

    assert!(allowed);
  }

  #[test]
  fn non_mcp_unknown_tools_are_rejected() {
    let allowed =
      ToolAllowList::is_tool_allowed_and_enabled(ToolId::Mcp("custom__tool".to_string()), AgentKind::Crew, false);

    assert!(!allowed);
  }

  #[test]
  fn non_mcp_subagent_restrictions_still_apply() {
    let allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::AskQuestion, AgentKind::Crew, true);
    assert!(!allowed);
  }

  #[test]
  fn non_planning_subagent_cannot_mutate_plans() {
    let create_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanCreate, AgentKind::Executor, true);
    let update_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanUpdate, AgentKind::Verifier, true);
    let delete_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanDelete, AgentKind::Designer, true);

    assert!(!create_allowed);
    assert!(!update_allowed);
    assert!(!delete_allowed);
  }

  #[test]
  fn planning_subagent_can_only_create_and_update_plans() {
    let create_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanCreate, AgentKind::Planner, true);
    let update_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanUpdate, AgentKind::Planner, true);
    let delete_allowed = ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanDelete, AgentKind::Planner, true);

    assert!(create_allowed);
    assert!(update_allowed);
    assert!(!delete_allowed);
  }

  #[test]
  fn memory_tools_are_disabled_when_runtime_gate_is_off() {
    assert!(!ToolAllowList::is_tool_allowed_and_enabled_for_runtime(
      ToolId::MemoryWrite,
      AgentKind::Planner,
      false,
      false,
    ));
    assert!(!ToolAllowList::is_tool_allowed_and_enabled_for_runtime(
      ToolId::MemorySearch,
      AgentKind::Researcher,
      false,
      false,
    ));
    assert!(ToolAllowList::is_tool_allowed_and_enabled_for_runtime(
      ToolId::FilesRead,
      AgentKind::Planner,
      false,
      false,
    ));
  }
}
