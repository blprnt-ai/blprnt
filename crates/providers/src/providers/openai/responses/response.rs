#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum OpenAiStreamEvent {
  // Lifecycle
  #[serde(rename = "response.created")]
  Created { response: ResponseSummary },
  #[serde(rename = "response.in_progress")]
  InProgress { response: ResponseSummary },
  #[serde(rename = "response.completed")]
  Completed { response: ResponseCompleted },
  #[serde(rename = "ping")]
  Ping { cost: Option<f32> },

  // Output Items belong to a response
  #[serde(rename = "response.output_item.added")]
  OutputItemAdded { output_index: u32, item: OutputItem },
  #[serde(rename = "response.output_item.done")]
  OutputItemDone { output_index: u32, item: OutputItem },

  // Content Parts belong to an output item
  #[serde(rename = "response.content_part.added")]
  ContentPartAdded {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    part:            ContentPart,
  },
  #[serde(rename = "response.content_part.done")]
  ContentPartDone {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    part:            ContentPart,
  },

  // Response
  #[serde(rename = "response.output_text.delta")]
  OutputTextDelta {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    delta:           String,
  },
  #[serde(rename = "response.output_text.done")]
  ResponseOutputTextDone {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    text:            String,
  },

  // I don't know if we need these
  #[serde(rename = "response.reasoning_text.delta")]
  ReasoningTextDelta {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    delta:           String,
  },
  #[serde(rename = "response.reasoning_text.done")]
  ReasoningTextDone {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    text:            String,
  },

  // Reasoning
  #[serde(rename = "response.reasoning_summary_part.added")]
  ReasoningSummaryPartAdded {
    item_id:         String,
    output_index:    u32,
    sequence_number: Option<u32>,
    part:            ReasoningContentPart,
  },
  #[serde(rename = "response.reasoning_summary_part.done")]
  ReasoningSummaryPartDone {
    item_id:         String,
    output_index:    u32,
    sequence_number: Option<u32>,
    part:            ReasoningContentPart,
  },

  #[serde(rename = "response.reasoning_summary_text.delta")]
  ReasoningSummaryTextDelta { item_id: String, output_index: u32, delta: String },
  #[serde(rename = "response.reasoning_summary_text.done")]
  ReasoningSummaryTextDone { item_id: String, output_index: u32, text: String },

  // Function Call
  #[serde(rename = "response.function_call_arguments.delta")]
  FunctionCallArgumentsDelta {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    delta:           String,
  },
  #[serde(rename = "response.function_call_arguments.done")]
  FunctionCallArgumentsDone {
    item_id:         String,
    output_index:    u32,
    content_index:   Option<u32>,
    sequence_number: Option<u32>,
    arguments:       String,
  },

  #[serde(rename = "response.output_text.annotation.added")]
  OutputTextAnnotationAdded {
    item_id:          String,
    output_index:     u32,
    content_index:    Option<u32>,
    sequence_number:  Option<u32>,
    annotation_index: Option<u32>,
    annotation:       Annotation,
  },

  #[serde(rename = "response.failed")]
  Failed { response: ResponseFailed },

  #[serde(rename = "keepalive")]
  KeepAlive { sequence_number: u32 },
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResponseSummary {
  pub id:         Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub object:     Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created_at: Option<i32>,
  pub status:     Option<String>,
  pub model:      Option<String>,
  #[serde(default)]
  pub output:     Vec<OutputItem>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub usage:      Option<Usage>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResponseStatusOnly {
  pub id:     Option<String>,
  pub status: ItemStatus,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResponseCompleted {
  pub id:         Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub object:     Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created_at: Option<i32>,
  pub status:     ItemStatus,
  pub model:      Option<String>,
  pub output:     Vec<OutputItem>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub usage:      Option<Usage>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Usage {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub input_tokens:  Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub output_tokens: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_tokens:  Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
  ReasoningText { text: String },
  OutputText { text: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
  Message {
    id:      Option<String>,
    content: Vec<MessageContentPart>,
  },
  FunctionCall {
    id:        Option<String>,
    name:      String,
    call_id:   String,
    arguments: String,
  },
  Reasoning {
    id:      Option<String>,
    content: Option<Vec<ReasoningContentPart>>,
    summary: Option<Vec<ReasoningContentPart>>,
  },
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
  #[default]
  Message,
  FunctionCall,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
  #[default]
  Completed,
  Incomplete,
  InProgress,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
  #[default]
  Assistant,
}

/// Message content parts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContentPart {
  OutputText { text: String },
  Refusal { refusal: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReasoningContentPart {
  pub text: String,
  #[serde(rename = "type")]
  pub kind: String,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Annotation {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url:         Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_index: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_index:   Option<usize>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResponseFailed {
  pub id:     String,
  pub status: FailedStatus,
  pub error:  StreamError,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailedStatus {
  #[default]
  Failed,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamError {
  pub message: String,
  pub code:    String,
  pub param:   Option<String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiCountTokensResponse {
  pub input_tokens: u32,
}
