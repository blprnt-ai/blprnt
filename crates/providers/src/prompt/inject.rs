use common::shared::prelude::*;

pub fn inject_user_prompt(user_prompt: QueueItem) -> Vec<MessageContent> {
  let mut middle: Vec<MessageContent> = user_prompt.into();

  let first = "<system-message>The user sent the following message.</system-message>".to_string();
  let last = "<system-message>Please address this message and continue with your tasks.</system-message>".to_string();

  middle.insert(0, MessageContent::Text(MessageText { text: first, signature: None }));
  middle.push(MessageContent::Text(MessageText { text: last, signature: None }));

  middle
}
