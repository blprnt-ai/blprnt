use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(
  title = "primer_get",
  description = "Retrieves the project's custom instructions that guide agent behavior. Do not use this tool unless you are explicitly editing the primer. The primer is already injected directly into your system prompt."
)]
pub struct GetPrimerArgs {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct GetPrimerPayload {
  pub content: String,
}

impl From<GetPrimerPayload> for ToolUseResponseData {
  fn from(payload: GetPrimerPayload) -> Self {
    Self::GetPrimer(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(
  title = "primer_update",
  description = "Updates the project's custom instructions that guide agent behavior."
)]
pub struct UpdatePrimerArgs {
  pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct UpdatePrimerPayload {
  pub content: String,
}

impl From<UpdatePrimerPayload> for ToolUseResponseData {
  fn from(payload: UpdatePrimerPayload) -> Self {
    Self::UpdatePrimer(payload)
  }
}
