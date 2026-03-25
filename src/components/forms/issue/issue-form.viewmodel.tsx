import { makeAutoObservable } from 'mobx'
import type { IssueDto } from '@/bindings/IssueDto'
import { issuesApi } from '@/lib/api/issues'
import { IssueModel } from '@/models/issue.model'

export class IssueFormViewmodel {
  public issue: IssueModel = new IssueModel()

  constructor() {
    makeAutoObservable(this)
  }

  public init = async (issueId?: string) => {
    if (!issueId) return

    const issue = await issuesApi.get(issueId)
    this.setIssue(issue)
  }

  private setIssue = (issue: IssueDto) => {
    this.issue = new IssueModel(issue)
  }

  public save = async () => {
    if (!this.issue.isDirty) return

    if (!this.issue.id) await this.createIssue()
    else await this.updateIssue()
  }

  private createIssue = async () => {
    const issue = await issuesApi.create(this.issue.toPayload())
    this.setIssue(issue)
  }

  private updateIssue = async () => {
    const issue = await issuesApi.update(this.issue.id!, this.issue.toPayloadPatch())
    this.setIssue(issue)
  }
}
