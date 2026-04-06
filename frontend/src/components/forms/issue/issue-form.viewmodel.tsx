import { makeAutoObservable, runInAction } from 'mobx'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssueLabel } from '@/bindings/IssueLabel'
import type { IssuePriority } from '@/bindings/IssuePriority'
import { colors } from '@/components/ui/colors'
import { issuesApi } from '@/lib/api/issues'
import { AppModel } from '@/models/app.model'
import { IssueModel } from '@/models/issue.model'

export class IssueFormViewmodel {
  public isOpen = false
  public isSaving = false
  public labelDraft = ''
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

  public get availableLabels(): IssueLabel[] {
    const labelMap = new Map<string, IssueLabel>()

    for (const issue of AppModel.instance.issues) {
      for (const label of issue.labels) {
        labelMap.set(label.name.toLowerCase(), label)
      }
    }

    return Array.from(labelMap.values()).sort((a, b) => a.name.localeCompare(b.name))
  }

  public get nextLabelColor() {
    return colors[this.availableLabels.length % colors.length]?.color ?? colors[0].color
  }

  private get hasDraft() {
    return Boolean(
      this.issue.title ||
        this.issue.description ||
        this.issue.project ||
        this.issue.assignee ||
        this.issue.priority !== 'medium' ||
        this.issue.labels.length > 0,
    )
  }

  public setLabelDraft(value: string) {
    this.labelDraft = value
  }

  public addLabel(name: string, color?: string) {
    const trimmed = name.trim()
    if (!trimmed) return

    const exists = this.issue.labels.some((label) => label.name.toLowerCase() === trimmed.toLowerCase())
    if (exists) return

    const nextColor = color ?? this.nextLabelColor
    this.issue.labels = [...this.issue.labels, { name: trimmed, color: nextColor }]
    this.labelDraft = ''
  }

  public removeLabel(name: string) {
    this.issue.labels = this.issue.labels.filter((label) => label.name !== name)
  }

  public open = () => {
    this.isOpen = true
  }

  public openWithDefaults = (defaults?: { assignee?: string; project?: string }) => {
    if (!this.hasDraft) {
      this.issue.assignee = defaults?.assignee ?? ''
      this.issue.project = defaults?.project ?? ''
    }

    this.isOpen = true
  }

  public cancel = () => {
    if (this.isSaving) return

    this.isOpen = false
    this.reset()
  }

  public dismiss = () => {
    if (this.isSaving) return

    this.isOpen = false
  }

  public setOpen = (isOpen: boolean) => {
    if (isOpen) {
      this.open()
      return
    }

    this.dismiss()
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
    this.labelDraft = ''
    this.issue = new IssueModel()
  }
}
