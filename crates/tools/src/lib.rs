#![allow(clippy::redundant_field_names)]
#![warn(unused, unused_crate_dependencies)]

mod file;
mod host;
mod rg;
mod skill;
mod tools;

pub(crate) mod tool_trait;
pub mod utils;

pub mod prelude;
pub mod tool_use;

use anyhow as _;
pub use cap_async_std::fs::Dir;
pub use shared::tools::ToolSpec;
pub use tool_trait::Tool;
pub use tools::Tools;
use tracing as _;

#[cfg(test)]
mod test {
  use shared::agent::AgentKind;
  use shared::tools::WorkingDirectories;
  use shared::tools::config::ToolsSchemaConfig;
  use tempdir as _;

  use super::*;

  #[test]
  fn test_tools_schema() {
    let tools_schema = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Crew,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });
    println!("Tools schema: {}", serde_json::to_string_pretty(&tools_schema).unwrap_or_default());
  }
}
