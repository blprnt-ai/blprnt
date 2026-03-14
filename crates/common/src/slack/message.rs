#[derive(Debug, Clone)]
pub struct SendMessage {
  pub content:   String,
  pub recipient: String,
  pub subject:   Option<String>,
  /// Platform thread identifier for threaded replies (e.g. Slack `thread_ts`).
  pub thread_ts: Option<String>,
}

impl SendMessage {
  /// Create a new message with content and recipient
  pub fn new(content: impl Into<String>, recipient: impl Into<String>) -> Self {
    Self { content: content.into(), recipient: recipient.into(), subject: None, thread_ts: None }
  }

  /// Create a new message with content, recipient, and subject
  pub fn with_subject(content: impl Into<String>, recipient: impl Into<String>, subject: impl Into<String>) -> Self {
    Self { content: content.into(), recipient: recipient.into(), subject: Some(subject.into()), thread_ts: None }
  }

  /// Set the thread identifier for threaded replies.
  pub fn in_thread(mut self, thread_ts: Option<String>) -> Self {
    self.thread_ts = thread_ts;
    self
  }
}

#[derive(Debug, Clone)]
pub struct ChannelMessage {
  pub id:           String,
  pub sender:       String,
  pub reply_target: String,
  pub content:      String,
  pub channel:      String,
  pub timestamp:    u64,
  /// Platform thread identifier (e.g. Slack `ts`, Discord thread ID).
  /// When set, replies should be posted as threaded responses.
  pub thread_ts:    Option<String>,
}
