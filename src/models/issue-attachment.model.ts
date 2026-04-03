import { flow, makeAutoObservable } from 'mobx'
import type { IssueAttachment } from '@/bindings/IssueAttachment'
import type { IssueAttachmentDetailDto } from '@/bindings/IssueAttachmentDetailDto'
import type { IssueAttachmentDto } from '@/bindings/IssueAttachmentDto'
import { issuesApi } from '@/lib/api/issues'
import { ModelField } from './model-field'

export class IssueAttachmentModel {
  public id: string
  private _attachment: ModelField<IssueAttachment>
  public creator: string
  public runId: string
  public createdAt: Date

  constructor(
    private readonly issueId: string,
    issueAttachment?: IssueAttachmentDto | IssueAttachmentDetailDto,
  ) {
    this.id = issueAttachment?.id ?? ''

    const detail = issueAttachment && 'attachment' in issueAttachment ? issueAttachment : null
    const summary = issueAttachment && !('attachment' in issueAttachment) ? issueAttachment : null
    const attachment: IssueAttachment = {
      attachment: detail?.attachment.attachment ?? '',
      attachment_kind: detail?.attachment.attachment_kind ?? summary?.attachment_kind ?? 'image',
      mime_kind: detail?.attachment.mime_kind ?? summary?.mime_kind ?? '',
      name: detail?.attachment.name ?? summary?.name ?? '',
      size: Number(detail?.attachment.size ?? summary?.size ?? 0),
    }

    this._attachment = new ModelField(attachment)
    this.creator = detail?.creator ?? ''
    this.runId = issueAttachment?.run_id ?? ''
    this.createdAt = new Date(issueAttachment?.created_at ?? '')

    makeAutoObservable(this)
  }

  public get attachment() {
    return this._attachment.value
  }

  public set attachment(attachment: IssueAttachment) {
    this._attachment.value = attachment
  }

  public hydrate(issueAttachment: IssueAttachmentDetailDto) {
    this.id = issueAttachment.id
    this.attachment = issueAttachment.attachment
    this.creator = issueAttachment.creator
    this.runId = issueAttachment.run_id ?? ''
    this.createdAt = new Date(issueAttachment.created_at)
  }

  public add = flow(function* (this: IssueAttachmentModel) {
    const payload = this._attachment.value
    const attachment = yield issuesApi.attachment(this.issueId, payload)
    this.id = attachment.id
    this.createdAt = new Date(attachment.created_at)
  })
}
