use surrealdb::types::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FallbackResponse {
  pub id:     String,
  pub error:  Option<FallbackError>,
  pub output: Vec<Output>,
  pub usage:  FallbackUsage,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FallbackError {
  pub message: String,
  pub code:    String,
  pub param:   Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
  Message(Message),
  Reasoning(Reasoning),
  FunctionCall(FunctionCall),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
  pub role:    String,
  pub content: Vec<MessageContent>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
  OutputText { id: String, text: String },
  Refusal { refusal: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Reasoning {
  pub id:      String,
  pub summary: Option<Vec<ReasoningContent>>,
  pub content: Option<Vec<ReasoningContent>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReasoningContent {
  OutputText { text: String },
  SummaryText { text: String },
  ReasoningText { text: String },
}

impl ReasoningContent {
  pub fn to_text(&self) -> String {
    match self {
      ReasoningContent::OutputText { text } => text.clone(),
      ReasoningContent::SummaryText { text } => text.clone(),
      ReasoningContent::ReasoningText { text } => text.clone(),
    }
  }
}

impl Default for Reasoning {
  fn default() -> Self {
    Self { id: Uuid::new_v7().to_string(), summary: None, content: None }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
  pub call_id:   String,
  pub name:      String,
  pub arguments: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FallbackUsage {
  pub input_tokens:  u32,
  pub output_tokens: u32,
  pub total_tokens:  u32,
}
