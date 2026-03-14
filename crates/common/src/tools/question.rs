use surrealdb_types::SurrealValue;

use crate::agent::ToolAllowList;
use crate::agent::ToolId;
use crate::tools::ToolSpec;
use crate::tools::ToolUseResponseData;
use crate::tools::config::ToolsSchemaConfig;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(title = "ask_question", description = "Presents a multiple-choice single-answer question to the user.")]
pub struct AskQuestionArgs {
  pub question: String,
  pub details:  String,
  pub options:  Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct AskQuestionPayload {
  pub answer: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AskQuestionAnswerSource {
  #[default]
  Desktop,
  SlackButton,
  SlackModal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AskQuestionClaimStatus {
  Accepted,
  RejectedAlreadyAnswered,
  Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct AskQuestionClaimResult {
  pub question_id:   String,
  pub answer_source: AskQuestionAnswerSource,
  pub outcome:       AskQuestionClaimStatus,
}

impl AskQuestionClaimResult {
  pub fn accepted(question_id: String, answer_source: AskQuestionAnswerSource) -> Self {
    Self { question_id, answer_source, outcome: AskQuestionClaimStatus::Accepted }
  }

  pub fn rejected_already_answered(question_id: String, answer_source: AskQuestionAnswerSource) -> Self {
    Self { question_id, answer_source, outcome: AskQuestionClaimStatus::RejectedAlreadyAnswered }
  }

  pub fn invalid(question_id: String, answer_source: AskQuestionAnswerSource) -> Self {
    Self { question_id, answer_source, outcome: AskQuestionClaimStatus::Invalid }
  }
}

impl From<AskQuestionPayload> for ToolUseResponseData {
  fn from(payload: AskQuestionPayload) -> Self {
    Self::AskQuestion(payload)
  }
}

impl AskQuestionArgs {
  pub fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::AskQuestion, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(AskQuestionArgs);
    let json = serde_json::to_value(&schema).expect("[AskQuestionArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[AskQuestionArgs] properties is required"),
      "required": json.get("required").expect("[AskQuestionArgs] required is required"),
    });

    let name = schema.get("title").expect("[AskQuestionArgs] title is required").clone();
    let description = schema.get("description").expect("[AskQuestionArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
