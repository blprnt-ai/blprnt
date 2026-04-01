import { flow, makeAutoObservable } from 'mobx'
import type { IssueCommentDto } from '@/bindings/IssueCommentDto'
import { issuesApi } from '@/lib/api/issues'
import { ModelField } from './model-field'

export class IssueCommentModel {
  public id: string
  private _comment: ModelField<string>
  public creator: string
  public runId: string
  public createdAt: Date

  constructor(
    private readonly issueId: string,
    issueComment?: IssueCommentDto,
  ) {
    this.id = issueComment?.id ?? ''
    this._comment = new ModelField(issueComment?.comment ?? '')
    this.creator = issueComment?.creator ?? ''
    this.runId = issueComment?.run_id ?? ''
    this.createdAt = new Date(issueComment?.created_at ?? '')

    makeAutoObservable(this)
  }

  public get isSaved() {
    return this.id !== ''
  }

  public get comment() {
    return this._comment.value
  }

  public set comment(comment: string) {
    this._comment.value = comment
  }

  public add = flow(function* (this: IssueCommentModel) {
    const payload = { comment: this._comment.value, reopen_issue: null }
    const comment = yield issuesApi.comment(this.issueId, payload)

    this.id = comment.id
    this.createdAt = new Date(comment.created_at)
  })
}
