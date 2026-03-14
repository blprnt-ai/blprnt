pub mod plan;
pub mod primer;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;

pub use self::plan::*;
pub use self::primer::*;
pub use crate::Tool;
pub use crate::ToolSpec;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Project {
  GetPrimer(GetPrimerTool),
  UpdatePrimer(UpdatePrimerTool),
  PlanCreate(PlanCreateTool),
  PlanList(PlanListTool),
  PlanGet(PlanGetTool),
  PlanUpdate(PlanUpdateTool),
  PlanDelete(PlanDeleteTool),
}

#[async_trait]
impl Tool for Project {
  fn tool_id(&self) -> ToolId {
    match self {
      Self::GetPrimer(_) => ToolId::PrimerGet,
      Self::UpdatePrimer(_) => ToolId::PrimerUpdate,
      Self::PlanCreate(_) => ToolId::PlanCreate,
      Self::PlanList(_) => ToolId::PlanList,
      Self::PlanGet(_) => ToolId::PlanGet,
      Self::PlanUpdate(_) => ToolId::PlanUpdate,
      Self::PlanDelete(_) => ToolId::PlanDelete,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Self::GetPrimer(tool) => tool.run(context).await,
      Self::UpdatePrimer(tool) => tool.run(context).await,
      Self::PlanCreate(tool) => tool.run(context).await,
      Self::PlanList(tool) => tool.run(context).await,
      Self::PlanGet(tool) => tool.run(context).await,
      Self::PlanUpdate(tool) => tool.run(context).await,
      Self::PlanDelete(tool) => tool.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();

    schema.extend(GetPrimerTool::schema(config));
    schema.extend(UpdatePrimerTool::schema(config));
    schema.extend(PlanCreateTool::schema(config));
    schema.extend(PlanListTool::schema(config));
    schema.extend(PlanGetTool::schema(config));
    schema.extend(PlanUpdateTool::schema(config));
    schema.extend(PlanDeleteTool::schema(config));

    schema
  }
}
