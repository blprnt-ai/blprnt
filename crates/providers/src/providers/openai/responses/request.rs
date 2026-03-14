use std::collections::HashMap;
use std::fmt::Display;

use common::models::ReasoningEffort;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponsesChatRequestBody {
  // For both responses and input_tokens
  pub input:               Vec<InputItem>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instructions:        Option<String>,
  pub model:               String,
  pub parallel_tool_calls: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools:               Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reasoning:           Option<ChatRequestBodyReasoning>,

  // Only for responses
  #[serde(skip_serializing_if = "Option::is_none")]
  pub store:  Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stream: Option<bool>,

  // let include: Vec<String> = if reasoning.is_some() {
  //     vec!["reasoning.encrypted_content".to_string()]
  // } else {
  //     vec![]
  // };
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub include: Vec<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatRequestBodyReasoning {
  pub effort:  CodexReasoningEffort,
  pub summary: Option<ReasoningSummary>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexReasoningEffort {
  #[serde(rename = "xhigh")]
  XHigh,
  High,
  #[default]
  Medium,
  Low,
  Minimal,
  None,
}

impl From<CodexReasoningEffort> for ChatRequestBodyReasoning {
  fn from(effort: CodexReasoningEffort) -> Self {
    Self { effort, summary: Some(ReasoningSummary::Detailed) }
  }
}

impl From<ReasoningEffort> for ChatRequestBodyReasoning {
  fn from(effort: ReasoningEffort) -> Self {
    match effort {
      ReasoningEffort::XHigh => {
        Self { effort: CodexReasoningEffort::XHigh, summary: Some(ReasoningSummary::Detailed) }
      }
      ReasoningEffort::High => Self { effort: CodexReasoningEffort::High, summary: Some(ReasoningSummary::Detailed) },
      ReasoningEffort::Medium => {
        Self { effort: CodexReasoningEffort::Medium, summary: Some(ReasoningSummary::Detailed) }
      }
      ReasoningEffort::Low => Self { effort: CodexReasoningEffort::Low, summary: Some(ReasoningSummary::Detailed) },
      ReasoningEffort::Minimal => {
        Self { effort: CodexReasoningEffort::Low, summary: Some(ReasoningSummary::Detailed) }
      }
      ReasoningEffort::None => Self { effort: CodexReasoningEffort::Low, summary: Some(ReasoningSummary::Detailed) },
    }
  }
}

impl Display for ChatRequestBodyReasoning {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.effort {
      CodexReasoningEffort::XHigh => write!(f, "xhigh"),
      CodexReasoningEffort::High => write!(f, "high"),
      CodexReasoningEffort::Medium => write!(f, "medium"),
      CodexReasoningEffort::Low => write!(f, "low"),
      CodexReasoningEffort::Minimal => write!(f, "minimal"),
      CodexReasoningEffort::None => write!(f, "none"),
    }
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummary {
  Auto,
  Concise,
  #[default]
  Detailed,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
  Message {
    role:    String,
    content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status:  Option<String>,
  },
  Reasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id:                Option<String>,
    summary:           Vec<ReasoningItemReasoningSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content:           Option<Vec<ReasoningItemContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_content: Option<String>,
  },
  LocalShellCall {
    /// Set when using the chat completions API.
    #[serde(skip_serializing)]
    id:      Option<String>,
    /// Set when using the Responses API.
    call_id: Option<String>,
    status:  LocalShellStatus,
    action:  LocalShellAction,
  },
  FunctionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    id:        Option<String>,
    name:      String,
    arguments: String,
    call_id:   String,
  },
  FunctionCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    id:      Option<String>,
    call_id: String,
    output:  String,
    status:  FunctionCallStatus,
  },
  CustomToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<String>,

    call_id: String,
    name:    String,
    input:   String,
  },
  CustomToolCallOutput {
    call_id: String,
    output:  String,
  },
  WebSearchCall {
    id:     String,
    status: String,
    action: String,
  },
}

impl Default for InputItem {
  fn default() -> Self {
    Self::Message { role: "user".to_string(), content: vec![], id: None, status: None }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentItem {
  InputText { text: String },
  InputImage { detail: InputImageDetail, image_url: String },
  OutputText(OutputText),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputImageDetail {
  Auto,
  High,
  Low,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OutputText {
  pub text:        String,
  pub annotations: Vec<String>,
}

impl OutputText {
  pub fn from_text(text: String) -> ContentItem {
    ContentItem::OutputText(Self { text, annotations: vec![] })
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReasoningItemReasoningSummary {
  SummaryText { text: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReasoningItemContent {
  ReasoningText { text: String },
  Text { text: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalShellStatus {
  Completed,
  InProgress,
  Incomplete,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalShellAction {
  Exec(LocalShellExecAction),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LocalShellExecAction {
  pub command:           Vec<String>,
  pub timeout_ms:        Option<u64>,
  pub working_directory: Option<String>,
  pub env:               Option<HashMap<String, String>>,
  pub user:              Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallStatus {
  Completed,
  InProgress,
  Incomplete,
}
