use serde::ser::SerializeStruct;
use serde_json::Value;
use surrealdb::types::ToSql;
use surrealdb::types::Uuid;
use surrealdb_types::SurrealValue;

use crate::agent::ToolId;
use crate::models::ReasoningEffort;
use crate::session_dispatch::events::SessionDispatchEvent;
use crate::shared::prelude::SurrealId;
use crate::shared::prelude::TokenUsage;
use crate::tools::ToolUseResponse;

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LlmEvent {
  CompactSummary(CompactSummary),

  // Reasoning
  ReasoningStarted(ReasoningStarted),
  Reasoning(ReasoningFinal),
  ReasoningDelta(ReasoningTextDelta),
  ReasoningDone(ReasoningDone),

  ReasoningEffortChanged(ReasoningEffortChanged),
  SkillApplied(SkillApplied),

  // Response
  ResponseStarted(ResponseStarted),
  Response(Response),
  ResponseDelta(ResponseDelta),
  ResponseDone(ResponseDone),

  // Tool use
  ToolCallStarted(ToolCallStarted),
  ToolCallCompleted(ToolCallCompleted),

  // Misc
  Status(Status),
  TokenUsage(TokenUsage),

  // Web search
  WebSearch(WebSearch),
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CompactSummary {
  #[specta(type = String)]
  pub id:      SurrealId,
  pub summary: String,
}

impl From<CompactSummary> for SessionDispatchEvent {
  fn from(value: CompactSummary) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::CompactSummary(value))
  }
}

impl serde::Serialize for CompactSummary {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("CompactSummary", 2)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("summary", &self.summary)?;
    state.end()
  }
}

// Reasoning

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningStarted {
  #[specta(type = String)]
  pub id:      SurrealId,
  #[specta(type = String)]
  pub turn_id: Uuid,
  #[specta(type = String)]
  pub step_id: Uuid,
}

impl From<ReasoningStarted> for SessionDispatchEvent {
  fn from(value: ReasoningStarted) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ReasoningStarted(value))
  }
}

impl serde::Serialize for ReasoningStarted {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ReasoningStarted", 3)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("turnId", &self.turn_id.to_string())?;
    state.serialize_field("stepId", &self.step_id.to_string())?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningFinal {
  #[specta(type = String)]
  pub id:        SurrealId,
  pub reasoning: String,
}

impl From<ReasoningFinal> for SessionDispatchEvent {
  fn from(value: ReasoningFinal) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::Reasoning(value))
  }
}

impl serde::Serialize for ReasoningFinal {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ReasoningFinal", 2)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("reasoning", &self.reasoning)?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningTextDelta {
  #[specta(type = String)]
  pub id:    SurrealId,
  #[specta(type = String)]
  pub delta: String,
}

impl From<ReasoningTextDelta> for SessionDispatchEvent {
  fn from(value: ReasoningTextDelta) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ReasoningDelta(value))
  }
}

impl serde::Serialize for ReasoningTextDelta {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ReasoningTextDelta", 2)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("delta", &self.delta)?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningDone {
  #[specta(type = String)]
  pub id: SurrealId,
}

impl From<ReasoningDone> for SessionDispatchEvent {
  fn from(value: ReasoningDone) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ReasoningDone(value))
  }
}

impl serde::Serialize for ReasoningDone {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ReasoningDone", 1)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.end()
  }
}

// Response

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResponseStarted {
  #[specta(type = String)]
  pub id:      SurrealId,
  #[specta(type = String)]
  pub turn_id: Uuid,
  #[specta(type = String)]
  pub step_id: Uuid,
}

impl From<ResponseStarted> for SessionDispatchEvent {
  fn from(value: ResponseStarted) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ResponseStarted(value))
  }
}

impl serde::Serialize for ResponseStarted {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ResponseStarted", 3)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("turnId", &self.turn_id.to_string())?;
    state.serialize_field("stepId", &self.step_id.to_string())?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Response {
  pub id:      SurrealId,
  pub content: String,
}

impl From<Response> for SessionDispatchEvent {
  fn from(value: Response) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::Response(value))
  }
}

impl serde::Serialize for Response {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("Response", 2)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("content", &self.content)?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResponseDelta {
  #[specta(type = String)]
  pub id:    SurrealId,
  pub delta: String,
}

impl From<ResponseDelta> for SessionDispatchEvent {
  fn from(value: ResponseDelta) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ResponseDelta(value))
  }
}

impl serde::Serialize for ResponseDelta {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ResponseDelta", 2)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.serialize_field("delta", &self.delta)?;
    state.end()
  }
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResponseDone {
  #[specta(type = String)]
  pub id: SurrealId,
}

impl From<ResponseDone> for SessionDispatchEvent {
  fn from(value: ResponseDone) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ResponseDone(value))
  }
}

impl serde::Serialize for ResponseDone {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("ResponseDone", 1)?;
    state.serialize_field("id", &self.id.0.to_sql())?;
    state.end()
  }
}

// Misc

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Status {
  pub status: String,
}

impl From<Status> for SessionDispatchEvent {
  fn from(value: Status) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::Status(value))
  }
}

impl serde::Serialize for Status {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("Status", 1)?;
    state.serialize_field("status", &self.status)?;
    state.end()
  }
}

impl Status {
  pub fn new(status: String) -> Self {
    Self { status }
  }
}

impl From<TokenUsage> for SessionDispatchEvent {
  fn from(value: TokenUsage) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::TokenUsage(value))
  }
}

impl TokenUsage {
  pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
    Self { input_tokens, output_tokens }
  }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallStarted {
  pub id:               String,
  #[specta(type = String)]
  pub turn_id:          Uuid,
  #[specta(type = String)]
  pub step_id:          Uuid,
  pub tool_id:          ToolId,
  pub args:             Value,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub question_id:      Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subagent_details: Option<SubagentDetails>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
#[serde(rename_all = "camelCase")]
pub struct SubagentDetails {
  pub session_id:        String,
  pub parent_session_id: Option<String>,
}

impl From<ToolCallStarted> for SessionDispatchEvent {
  fn from(value: ToolCallStarted) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ToolCallStarted(value))
  }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallCompleted {
  pub id:      String,
  pub item_id: String,
  pub content: ToolUseResponse,
}

impl From<ToolCallCompleted> for SessionDispatchEvent {
  fn from(value: ToolCallCompleted) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ToolCallCompleted(value))
  }
}

// Web search

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WebSearch {
  pub id:         String,
  pub web_search: WebSearchData,
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchData {
  pub url:         String,
  pub title:       String,
  pub start_index: usize,
  pub end_index:   usize,
}

impl From<WebSearch> for SessionDispatchEvent {
  fn from(value: WebSearch) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::WebSearch(value))
  }
}

impl WebSearch {
  pub fn new(id: String, web_search: WebSearchData) -> Self {
    Self { id, web_search }
  }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningEffortChanged {
  pub effort: ReasoningEffort,
}

impl From<ReasoningEffortChanged> for SessionDispatchEvent {
  fn from(value: ReasoningEffortChanged) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::ReasoningEffortChanged(value))
  }
}

impl ReasoningEffortChanged {
  pub fn new(effort: ReasoningEffort) -> Self {
    Self { effort }
  }
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SkillApplied {
  pub skill: String,
}

impl From<SkillApplied> for SessionDispatchEvent {
  fn from(value: SkillApplied) -> Self {
    SessionDispatchEvent::Llm(LlmEvent::SkillApplied(value))
  }
}

impl SkillApplied {
  pub fn new(skill: String) -> Self {
    Self { skill }
  }
}
