import { makeAutoObservable, runInAction } from 'mobx'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssuePriority } from '@/bindings/IssuePriority'
import { issuesApi } from '@/lib/api/issues'
import { AppModel } from '@/models/app.model'
import { IssueModel } from '@/models/issue.model'

export class IssueFormViewmodel {
  public isOpen = false
  public isSaving = false
  public issue: IssueModel = new IssueModel()
  private onCreated?: (issue: IssueDto) => Promise<void> | void

  constructor(onCreated?: (issue: IssueDto) => Promise<void> | void) {
    this.onCreated = onCreated
    makeAutoObservable(this)
  }

  public get canSave() {
    return this.issue.isValid && !this.isSaving
  }

  public get projectOptions() {
    return AppModel.instance.projects.map((project) => ({
      label: project.name,
      value: project.id,
    }))
  }

  public get assigneeOptions() {
    return AppModel.instance.employees.map((employee) => ({
      label: employee.role === 'owner' ? 'You' : employee.name,
      value: employee.id,
    }))
  }

  public get priorityOptions(): Array<{ label: string; value: IssuePriority }> {
    return [
      { label: 'Low', value: 'low' },
      { label: 'Medium', value: 'medium' },
      { label: 'High', value: 'high' },
      { label: 'Critical', value: 'critical' },
    ]
  }

  public open = () => {
    this.openWithDefaults()
  }

  public openWithDefaults = (defaults?: { assignee?: string; project?: string }) => {
    this.reset()
    this.issue.assignee = defaults?.assignee ?? ''
    this.issue.project = defaults?.project ?? ''
    this.isOpen = true
  }

  public close = () => {
    if (this.isSaving) return
    this.isOpen = false
    this.reset()
  }

  public setOpen = (isOpen: boolean) => {
    if (isOpen) {
      this.open()
      return
    }

    this.close()
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
    if (!this.issue.isValid || this.isSaving) return null
    if (this.issue.id) return null

    this.isSaving = true

    try {
      const issue = await issuesApi.create(this.issue.toPayload())
      await this.onCreated?.(issue)

      runInAction(() => {
        this.isOpen = false
        this.reset()
      })

      return issue
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  private reset = () => {
    this.issue = new IssueModel()
  }
}
