import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { Employee } from '@/bindings/Employee'
import type { IssueAttachment } from '@/bindings/IssueAttachment'
import type { IssuePatchPayload } from '@/bindings/IssuePatchPayload'
import { issuesApi } from '@/lib/api/issues'
import { connectIssueStream } from '@/lib/api/issues-stream'
import { AppModel } from '@/models/app.model'
import { IssueModel } from '@/models/issue.model'
import { IssueActionModel } from '@/models/issue-action.model'
import { IssueAttachmentModel } from '@/models/issue-attachment.model'
import { IssueCommentModel } from '@/models/issue-comment.model'
import { RunSummaryModel } from '@/models/run-summary.model'
import {
  filterMentionSuggestions,
  getMentionQuery,
  insertMentionSelection,
  type MentionSelection,
  mentionPayloadsFromSelections,
  reconcileMentionSelections,
} from './comment-mentions'

export interface IssueAttachmentUploadFile {
  name: string
  size: number
  type: string
}

type AttachmentReader = (file: IssueAttachmentUploadFile) => Promise<string>

const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024
const MAX_ATTACHMENT_BATCH_BYTES = 25 * 1024 * 1024

export class IssueViewmodel {
  public issue: IssueModel | null = null
  public parentIssue: IssueModel | null = null
  public childIssues: IssueModel[] = []
  public runs: RunSummaryModel[] = []
  public isLoading = true
  public isLoadingChildIssues = false
  public isSavingMetadata = false
  public isSavingTitle = false
  public isSavingDescription = false
  public isSubmittingComment = false
  public isUploadingAttachments = false
  public commentDraft = ''
  public commentCursor = 0
  public commentMentions: MentionSelection[] = []
  public reopenIssueOnComment = true
  public errorMessage: string | null = null
  public childIssuesErrorMessage: string | null = null
  private readonly issueId: string
  private readonly employeeId: string
  private readonly readAttachment: AttachmentReader
  private socket: WebSocket | null = null

  constructor(issueId: string, employeeId: string, readAttachment: AttachmentReader = readAttachmentAsDataUrl) {
    this.issueId = issueId
    this.employeeId = employeeId
    this.readAttachment = readAttachment

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get canSubmitComment() {
    return Boolean(this.issue?.id) && this.commentDraft.trim().length > 0 && !this.isSubmittingComment
  }

  public get canSaveMetadata() {
    return Boolean(this.issue?.id) && Boolean(this.issue?.isDirty) && !this.isSavingMetadata
  }

  public get activeMentionQuery() {
    return getMentionQuery(this.commentDraft, this.commentCursor)
  }

  public get mentionSuggestions(): Employee[] {
    const activeQuery = this.activeMentionQuery
    if (!activeQuery) return []
    return filterMentionSuggestions(AppModel.instance.employees, activeQuery.query)
  }

  public get timelineItems() {
    const comments = (this.issue?.comments ?? []).map((comment) => ({ type: 'comment' as const, createdAt: comment.createdAt, comment }))
    const runs = this.runs.map((run) => ({ type: 'run' as const, createdAt: run.createdAt, run }))

    return [...comments, ...runs].sort((left, right) => left.createdAt.getTime() - right.createdAt.getTime())
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
      this.childIssuesErrorMessage = null
    })

    try {
      const issue = await issuesApi.get(this.issueId)
      runInAction(() => {
        this.issue = new IssueModel(issue)
      })
      await this.hydrateAttachments()
      await Promise.all([this.loadRuns(), this.loadChildIssues(), this.loadParentIssue()])
      this.connect()
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load this issue.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public async loadChildIssues() {
    runInAction(() => {
      this.isLoadingChildIssues = true
      this.childIssuesErrorMessage = null
    })

    try {
      const childIssues = await issuesApi.listChildren(this.issueId)
      runInAction(() => {
        this.childIssues = childIssues.map((issue) => new IssueModel(issue))
      })
    } catch (error) {
      runInAction(() => {
        this.childIssues = []
        this.childIssuesErrorMessage = getErrorMessage(error, 'Unable to load child issues.')
      })
    } finally {
      runInAction(() => {
        this.isLoadingChildIssues = false
      })
    }
  }

  public async loadParentIssue() {
    if (!this.issue?.parent) return

    try {
      const parentIssue = await issuesApi.get(this.issue.parent)
      runInAction(() => {
        this.parentIssue = new IssueModel(parentIssue)
      })
    } catch {}
  }

  public async loadRuns() {
    try {
      const runs = await issuesApi.listRuns(this.issueId)
      runInAction(() => {
        this.runs = runs.map((run) => new RunSummaryModel(run))
      })
    } catch {
      runInAction(() => {
        this.runs = []
      })
    }
  }

  public setCommentDraft(comment: string, cursor = comment.length) {
    this.commentDraft = comment
    this.commentCursor = cursor
    this.commentMentions = reconcileMentionSelections(comment, this.commentMentions)
  }

  public selectCommentMention(employee: Employee) {
    const activeQuery = this.activeMentionQuery
    if (!activeQuery) return null

    const { nextCaret, nextText, selection } = insertMentionSelection(this.commentDraft, activeQuery, employee)
    this.commentDraft = nextText
    this.commentCursor = nextCaret
    this.commentMentions = reconcileMentionSelections(nextText, [...this.commentMentions, selection])

    return nextCaret
  }

  public setReopenIssueOnComment(shouldReopen: boolean) {
    this.reopenIssueOnComment = shouldReopen
  }

  public disconnect() {
    if (this.socket) this.socket.close()
    this.socket = null
  }

  public async saveMetadata() {
    if (!this.issue?.id || !this.issue.isDirty) return null

    runInAction(() => {
      this.isSavingMetadata = true
      this.errorMessage = null
    })

    try {
      const issue = await issuesApi.update(this.issue.id, this.issue.toPayloadPatch())
      runInAction(() => {
        this.issue = new IssueModel(issue)
      })

      return this.issue
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save issue metadata.')
      })

      return null
    } finally {
      runInAction(() => {
        this.isSavingMetadata = false
      })
    }
  }

  public async saveTitle(title: string) {
    return this.saveIssueFields({ title }, 'Unable to save the issue title.', 'isSavingTitle')
  }

  public async saveDescription(description: string) {
    return this.saveIssueFields({ description }, 'Unable to save the issue description.', 'isSavingDescription')
  }

  public async submitComment() {
    if (!this.issue?.id || this.commentDraft.trim().length === 0) return null
    const shouldReopenIssue = this.reopenIssueOnComment && this.issue.status === 'done'

    runInAction(() => {
      this.isSubmittingComment = true
      this.errorMessage = null
    })

    try {
      const comment = await issuesApi.comment(this.issue.id, {
        comment: this.commentDraft.trim(),
        mentions: mentionPayloadsFromSelections(this.commentMentions),
        reopen_issue: shouldReopenIssue,
      })
      const nextComment = new IssueCommentModel(this.issue.id, comment)

      runInAction(() => {
        this.issue?.addComment(nextComment)
        this.issue?.addAction(
          new IssueActionModel({
            action_kind: 'add_comment',
            created_at: nextComment.createdAt.toISOString(),
            creator: nextComment.creator,
            id: nextComment.id || crypto.randomUUID(),
            run_id: nextComment.runId || null,
          }),
        )
        if (shouldReopenIssue && this.issue) {
          this.issue.status = 'todo'
        }
        this.commentDraft = ''
        this.commentCursor = 0
        this.commentMentions = []
      })

      if (shouldReopenIssue) {
        toast.success('Comment posted and issue reopened.')
      }

      return nextComment
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to add your comment.')
      })

      return null
    } finally {
      runInAction(() => {
        this.isSubmittingComment = false
      })
    }
  }

  public async addAttachments(files: IssueAttachmentUploadFile[]) {
    if (!this.issue?.id || files.length === 0) return []

    const attachmentValidationError = validateAttachmentBatch(files)
    if (attachmentValidationError) {
      runInAction(() => {
        this.errorMessage = attachmentValidationError
      })
      return []
    }

    runInAction(() => {
      this.isUploadingAttachments = true
      this.errorMessage = null
    })

    try {
      const attachments: IssueAttachmentModel[] = []

      for (const file of files) {
        const attachment = new IssueAttachmentModel(this.issue.id)

        attachment.attachment = {
          attachment: await this.readAttachment(file),
          attachment_kind: file.type.startsWith('image/') ? 'image' : 'file',
          mime_kind: file.type,
          name: file.name,
          size: file.size,
        }

        await attachment.add()

        runInAction(() => {
          this.issue?.addAttachment(attachment)
          this.issue?.addAction(
            new IssueActionModel({
              action_kind: 'add_attachment',
              created_at: attachment.createdAt.toISOString(),
              creator: 'You',
              id: attachment.id || crypto.randomUUID(),
              run_id: attachment.runId || null,
            }),
          )
          attachments.push(attachment)
        })
      }

      return attachments
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to upload those attachments.')
      })

      return []
    } finally {
      runInAction(() => {
        this.isUploadingAttachments = false
      })
    }
  }

  private async saveIssueFields(
    patch: IssuePatchPayload,
    fallbackErrorMessage: string,
    savingState: 'isSavingTitle' | 'isSavingDescription',
  ) {
    if (!this.issue?.id) return null

    runInAction(() => {
      this[savingState] = true
      this.errorMessage = null
    })

    try {
      const issue = await issuesApi.update(this.issue.id, patch)
      runInAction(() => {
        this.issue = new IssueModel(issue)
      })

      return this.issue
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, fallbackErrorMessage)
      })

      return null
    } finally {
      runInAction(() => {
        this[savingState] = false
      })
    }
  }

  private connect() {
    this.disconnect()
    this.socket = connectIssueStream(this.employeeId, {
      onMessage: (message) => {
        runInAction(() => {
          if (message.type !== 'upsert') return

          if (message.issue.id === this.issueId) {
            this.issue = new IssueModel(message.issue)
            void this.loadRuns()
          }

          const isChildOfCurrentIssue = message.issue.parent_id === this.issueId
          const hasExistingChild = this.childIssues.some((issue) => issue.id === message.issue.id)

          if (isChildOfCurrentIssue) {
            const nextChildIssue = new IssueModel(message.issue)
            this.childIssues = hasExistingChild
              ? this.childIssues.map((issue) => (issue.id === nextChildIssue.id ? nextChildIssue : issue))
              : [...this.childIssues, nextChildIssue]
            return
          }

          if (hasExistingChild) {
            this.childIssues = this.childIssues.filter((issue) => issue.id !== message.issue.id)
          }
        })
      },
    })
  }

  private async hydrateAttachments() {
    if (!this.issue?.id || this.issue.attachments.length === 0) return

    const attachmentResults = await Promise.allSettled(
      this.issue.attachments.map((attachment) => issuesApi.getAttachment(this.issue!.id!, attachment.id)),
    )

    runInAction(() => {
      attachmentResults.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          this.issue?.attachments[index]?.hydrate(result.value)
        }
      })
    })
  }
}

const readAttachmentAsDataUrl = async (file: IssueAttachmentUploadFile) => {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader()

    reader.addEventListener('load', () => {
      if (typeof reader.result === 'string') {
        resolve(reader.result)
        return
      }

      reject(new Error('Unable to read attachment contents.'))
    })

    reader.addEventListener('error', () => {
      reject(reader.error ?? new Error('Unable to read attachment contents.'))
    })

    reader.readAsDataURL(file as unknown as Blob)
  })
}

const getErrorMessage = (error: unknown, fallback: string) => {
  return error instanceof Error ? error.message : fallback
}

const validateAttachmentBatch = (files: IssueAttachmentUploadFile[]) => {
  const oversizedFile = files.find((file) => file.size > MAX_ATTACHMENT_BYTES)
  if (oversizedFile) {
    return `${oversizedFile.name} is larger than 10 MB.`
  }

  const totalBytes = files.reduce((sum, file) => sum + file.size, 0)
  if (totalBytes > MAX_ATTACHMENT_BATCH_BYTES) {
    return 'Attachment batch is larger than 25 MB.'
  }

  return null
}

export const toIssueAttachmentPayload = async (
  file: IssueAttachmentUploadFile,
  readAttachment: AttachmentReader,
): Promise<IssueAttachment> => {
  return {
    attachment: await readAttachment(file),
    attachment_kind: file.type.startsWith('image/') ? 'image' : 'file',
    mime_kind: file.type,
    name: file.name,
    size: file.size,
  }
}

export const IssueViewmodelContext = createContext<IssueViewmodel | null>(null)

export const useIssueViewmodel = () => {
  const issueViewmodel = useContext(IssueViewmodelContext)
  if (!issueViewmodel) {
    throw new Error('useIssueViewmodel must be used within a IssueViewmodelProvider')
  }
  return issueViewmodel
}
