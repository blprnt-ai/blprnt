import { flow } from 'mobx'
import type { IssueAttachment } from '@/bindings/IssueAttachment'
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
    issueAttachment?: IssueAttachmentDto,
  ) {
    this.id = issueAttachment?.id ?? ''

    const attachment = issueAttachment?.attachment ?? {
      attachment: '',
      attachment_kind: 'Image',
      mime_kind: '',
      name: '',
      size: 0,
    }
    this._attachment = new ModelField(attachment)
    this.creator = issueAttachment?.creator ?? ''
    this.runId = issueAttachment?.run_id ?? ''
    this.createdAt = new Date(issueAttachment?.created_at ?? '')
  }

  public get attachment() {
    return this._attachment.value
  }

  public set attachment(attachment: IssueAttachment) {
    this._attachment.value = attachment
  }

  public add = flow(function* (this: IssueAttachmentModel) {
    const payload = this._attachment.value
    const attachment = yield issuesApi.attachment(this.issueId, payload)
    this.id = attachment.id
    this.createdAt = new Date(attachment.created_at)
  })
}
