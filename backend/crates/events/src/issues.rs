use persistence::prelude::IssueId;

use crate::bus::Events;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IssueEventKind {
  Created,
  Updated,
  CommentAdded,
  AttachmentAdded,
  Assigned,
  Unassigned,
  CheckedOut,
  Released,
}

#[derive(Clone, Debug)]
pub struct IssueEvent {
  pub issue_id: IssueId,
  pub kind:     IssueEventKind,
}

lazy_static::lazy_static! {
  pub static ref ISSUE_EVENTS: Events<IssueEvent> = Events::new();
}
