import type { IssueActionDto } from '@/bindings/IssueActionDto'
import type { IssueActionKind } from '@/bindings/IssueActionKind'

export class IssueActionModel {
  public id: string
  public action: IssueActionKind
  public creator: string
  public runId: string
  public createdAt: Date

  constructor(issueAction?: IssueActionDto) {
    this.id = issueAction?.id ?? ''
    this.action = issueAction?.action_kind ?? 'Create'
    this.creator = issueAction?.creator ?? ''
    this.runId = issueAction?.run_id ?? ''
    this.createdAt = new Date(issueAction?.created_at ?? '')
  }
}
