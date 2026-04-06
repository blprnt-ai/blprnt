import { makeAutoObservable } from 'mobx'
import type { CreateIssuePayload } from '@/bindings/CreateIssuePayload'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssueLabel } from '@/bindings/IssueLabel'
import type { IssuePatchPayload } from '@/bindings/IssuePatchPayload'
import type { IssuePriority } from '@/bindings/IssuePriority'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { IssueActionModel } from './issue-action.model'
import { IssueAttachmentModel } from './issue-attachment.model'
import { IssueCommentModel } from './issue-comment.model'
import { ModelField } from './model-field'

export class IssueModel {
  public id: string | null
  public identifier: string
  private _title: ModelField<string>
  private _description: ModelField<string>
  private _labels: ModelField<IssueLabel[]>
  private _status: ModelField<IssueStatus>
  private _project: ModelField<string>
  private _assignee: ModelField<string>
  private _blockedBy: ModelField<string>
  private _priority: ModelField<IssuePriority>
  private _checkedOutBy: ModelField<string>

  public parent: string
  public creator: string
  public createdAt: Date
  public updatedAt: Date

  public comments: IssueCommentModel[] = []
  public attachments: IssueAttachmentModel[] = []
  public actions: IssueActionModel[] = []

  constructor(issue?: IssueDto) {
    this.id = issue?.id ?? null
    this.identifier = issue?.identifier ?? ''
    this._title = new ModelField(issue?.title ?? '')
    this._description = new ModelField(issue?.description ?? '')
    this._labels = new ModelField(issue?.labels ?? [])
    this._status = new ModelField(issue?.status ?? 'todo')
    this._project = new ModelField(issue?.project ?? '')
    this._priority = new ModelField(issue?.priority ?? 'medium')
    this._assignee = new ModelField(issue?.assignee ?? '')
    this._blockedBy = new ModelField(issue?.blocked_by ?? '')
    this._checkedOutBy = new ModelField(issue?.checked_out_by ?? '')

    this.parent = issue?.parent_id ?? ''
    this.creator = issue?.creator ?? ''
    this.createdAt = new Date(issue?.created_at ?? '')
    this.updatedAt = new Date(issue?.updated_at ?? '')

    if (issue?.id) {
      this.comments = issue.comments.map((comment) => new IssueCommentModel(issue.id, comment)) ?? []
      this.attachments = issue.attachments.map((attachment) => new IssueAttachmentModel(issue.id, attachment)) ?? []
      this.actions = issue?.actions.map((action) => new IssueActionModel(action)) ?? []
    }

    makeAutoObservable(this)
  }

  public get isDirty() {
    return (
      this._title.isDirty ||
      this._description.isDirty ||
      this._labels.isDirty ||
      this._status.isDirty ||
      this._project.isDirty ||
      this._priority.isDirty ||
      this._assignee.isDirty ||
      this._blockedBy.isDirty ||
      this._checkedOutBy.isDirty
    )
  }

  public get isValid() {
    return this.title.trim().length > 0 && this.description.trim().length > 0
  }

  public clearDirty() {
    this._title.clearDirty()
    this._description.clearDirty()
    this._labels.clearDirty()
    this._status.clearDirty()
    this._project.clearDirty()
    this._priority.clearDirty()
    this._assignee.clearDirty()
    this._blockedBy.clearDirty()
    this._checkedOutBy.clearDirty()
  }

  public get title() {
    return this._title.value
  }

  public set title(title: string) {
    this._title.value = title
  }

  public get description() {
    return this._description.value
  }

  public set description(description: string) {
    this._description.value = description
  }

  public get labels() {
    return this._labels.value
  }

  public set labels(labels: IssueLabel[]) {
    this._labels.value = labels
  }

  public get status() {
    return this._status.value
  }

  public set status(status: IssueStatus) {
    this._status.value = status
  }

  public get project() {
    return this._project.value
  }

  public set project(project: string) {
    this._project.value = project
  }

  public setProject(project: string) {
    this._project.value = project
  }

  public get assignee() {
    return this._assignee.value
  }

  public set assignee(assignee: string) {
    this._assignee.value = assignee
  }

  public setAssignee(assignee: string) {
    this._assignee.value = assignee
  }

  public get blockedBy() {
    return this._blockedBy.value
  }

  public set blockedBy(blockedBy: string) {
    this._blockedBy.value = blockedBy
  }

  public get checkedOutBy() {
    return this._checkedOutBy.value
  }

  public set checkedOutBy(checkedOutBy: string) {
    this._checkedOutBy.value = checkedOutBy
  }

  public get priority() {
    return this._priority.value
  }

  public set priority(priority: IssuePriority) {
    this._priority.value = priority
  }

  public addAttachment(attachment: IssueAttachmentModel) {
    const existingAttachmentIndex = this.attachments.findIndex((existingAttachment) => existingAttachment.id === attachment.id)
    if (attachment.id && existingAttachmentIndex >= 0) {
      this.attachments[existingAttachmentIndex] = attachment
      return
    }

    this.attachments.push(attachment)
  }

  public addAction(action: IssueActionModel) {
    const existingActionIndex = this.actions.findIndex((existingAction) => existingAction.id === action.id)
    if (action.id && existingActionIndex >= 0) {
      this.actions[existingActionIndex] = action
      return
    }

    this.actions.push(action)
  }

  public addComment(comment: IssueCommentModel) {
    const existingCommentIndex = this.comments.findIndex((existingComment) => {
      if (comment.id && existingComment.id) {
        return existingComment.id === comment.id
      }

      return (
        existingComment.creator === comment.creator &&
        existingComment.comment === comment.comment &&
        existingComment.createdAt.getTime() === comment.createdAt.getTime()
      )
    })

    if (existingCommentIndex >= 0) {
      this.comments[existingCommentIndex] = comment
      return
    }

    this.comments.push(comment)
  }

  public toPayload(): CreateIssuePayload {
    return {
      assignee: this.assignee || null,
      description: this.description,
      labels: this.labels.length > 0 ? this.labels : undefined,
      parent: this.parent || null,
      priority: this.priority,
      project: this.project || null,
      status: this.status,
      title: this.title,
    }
  }

  public toPayloadPatch(): IssuePatchPayload {
    const assignee = this._assignee.dirtyValue === '' ? null : (this._assignee.dirtyValue ?? undefined)
    const blockedBy = this._blockedBy.dirtyValue === '' ? null : (this._blockedBy.dirtyValue ?? undefined)
    const project = this._project.dirtyValue === '' ? null : (this._project.dirtyValue ?? undefined)

    return {
      assignee: assignee,
      blocked_by: blockedBy,
      description: this._description.dirtyValue ?? undefined,
      labels: this._labels.dirtyValue ?? undefined,
      priority: this._priority.dirtyValue ?? undefined,
      project: project,
      status: this._status.dirtyValue ?? undefined,
      title: this._title.dirtyValue ?? undefined,
    }
  }
}
