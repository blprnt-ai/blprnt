import type { IssuePriority } from './IssuePriority'
import type { IssueStatus } from './IssueStatus'
import type { MyWorkReasonDto } from './MyWorkReasonDto'

export interface MyWorkItemDto {
  issue_id: string
  issue_identifier: string
  title: string
  status: IssueStatus
  priority: IssuePriority
  project_id: string | null
  project_name: string | null
  reason: MyWorkReasonDto
  relevant_at: string
  comment_id: string | null
  comment_snippet: string | null
}