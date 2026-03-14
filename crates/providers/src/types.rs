#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ChatBasic {
  pub messages: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParsedContentBlock {
  pub index:         u32,
  pub content_block: ContentBlock,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
  Text { id: String, text: String, signature: Option<String> },
  Thinking { id: String, thinking: String, signature: Option<String> },
  ToolUse { id: String, name: String, input: String, signature: Option<String> },
  Status { id: String, status: String, signature: Option<String> },
}

impl ParsedContentBlock {
  pub fn get_id(&self) -> String {
    match &self.content_block {
      ContentBlock::Text { id, .. } => id.clone(),
      ContentBlock::Thinking { id, .. } => id.clone(),
      ContentBlock::ToolUse { id, .. } => id.clone(),
      ContentBlock::Status { status, .. } => status.clone(),
    }
  }

  pub fn get_signature(&self) -> Option<String> {
    match &self.content_block {
      ContentBlock::Text { signature, .. } => signature.clone(),
      ContentBlock::Thinking { signature, .. } => signature.clone(),
      ContentBlock::ToolUse { signature, .. } => signature.clone(),
      ContentBlock::Status { signature, .. } => signature.clone(),
    }
  }

  pub fn set_signature(&mut self, new_signature: Option<String>) {
    match &mut self.content_block {
      ContentBlock::Text { signature, .. } => *signature = new_signature,
      ContentBlock::Thinking { signature, .. } => *signature = new_signature,
      ContentBlock::ToolUse { signature, .. } => *signature = new_signature,
      ContentBlock::Status { signature, .. } => *signature = new_signature,
    }
  }

  pub fn append_signature(&mut self, new_signature: String) {
    let signature = match &mut self.content_block {
      ContentBlock::Text { signature, .. } => {
        signature.clone().map_or(new_signature.clone(), |s| format!("{}{}", s, new_signature))
      }
      ContentBlock::Thinking { signature, .. } => {
        signature.clone().map_or(new_signature.clone(), |s| format!("{}{}", s, new_signature))
      }
      ContentBlock::ToolUse { signature, .. } => {
        signature.clone().map_or(new_signature.clone(), |s| format!("{}{}", s, new_signature))
      }
      ContentBlock::Status { signature, .. } => {
        signature.clone().map_or(new_signature.clone(), |s| format!("{}{}", s, new_signature))
      }
    };
    self.set_signature(Some(signature));
  }

  pub fn new_text(index: u32, id: String, signature: Option<String>) -> Self {
    Self { index, content_block: ContentBlock::Text { id, text: String::new(), signature } }
  }

  pub fn new_function_call(index: u32, id: String, name: String) -> Self {
    Self { index, content_block: ContentBlock::ToolUse { id, name, input: String::new(), signature: None } }
  }

  pub fn get_text(&self) -> String {
    match &self.content_block {
      ContentBlock::Text { text, .. } => text.clone(),
      _ => unreachable!(),
    }
  }

  pub fn set_text(&mut self, text: String) {
    self.content_block = ContentBlock::Text { id: self.get_id(), text, signature: self.get_signature() };
  }

  pub fn new_status(index: u32, id: String, status: String, signature: Option<String>) -> Self {
    Self { index, content_block: ContentBlock::Status { id, status, signature } }
  }

  pub fn get_status(&self) -> String {
    match &self.content_block {
      ContentBlock::Status { status, .. } => status.clone(),
      _ => unreachable!(),
    }
  }

  pub fn set_status(&mut self, status: String) {
    self.content_block = ContentBlock::Status { id: self.get_id(), status, signature: self.get_signature() };
  }

  pub fn append_status(&mut self, status: String) {
    let status = format!("{}{}", self.get_status(), status);
    self.set_status(status);
  }

  pub fn append_text(&mut self, text: String) {
    let text = format!("{}{}", self.get_text(), text);
    self.set_text(text);
  }

  pub fn new_thinking(index: u32, id: String, signature: Option<String>) -> Self {
    Self { index, content_block: ContentBlock::Thinking { id, thinking: String::new(), signature } }
  }

  pub fn get_thinking(&self) -> String {
    match &self.content_block {
      ContentBlock::Thinking { thinking, .. } => thinking.clone(),
      _ => unreachable!(),
    }
  }

  pub fn set_thinking(&mut self, thinking: String) {
    self.content_block = ContentBlock::Thinking { id: self.get_id(), thinking, signature: self.get_signature() };
  }

  pub fn append_thinking(&mut self, thinking: String) {
    let thinking = format!("{}{}", self.get_thinking(), thinking);
    self.set_thinking(thinking);
  }

  pub fn new_tool_use(index: u32, id: String, name: String, signature: Option<String>) -> Self {
    Self { index, content_block: ContentBlock::ToolUse { id, name, input: String::new(), signature } }
  }

  pub fn get_name(&self) -> String {
    match &self.content_block {
      ContentBlock::ToolUse { name, .. } => name.clone(),
      _ => unreachable!(),
    }
  }

  pub fn get_input(&self) -> String {
    match &self.content_block {
      ContentBlock::ToolUse { input, .. } => input.clone(),
      _ => unreachable!(),
    }
  }

  pub fn set_input(&mut self, input: String) {
    self.content_block =
      ContentBlock::ToolUse { id: self.get_id(), name: self.get_name(), input, signature: self.get_signature() };
  }

  pub fn append_input(&mut self, input: String) {
    let input = format!("{}{}", self.get_input(), input);
    self.set_input(input);
  }
}
