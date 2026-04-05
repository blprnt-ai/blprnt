import type { AddCommentPayload } from '@/bindings/AddCommentPayload'
import type { AssignIssuePayload } from '@/bindings/AssignIssuePayload'
import type { CreateIssuePayload } from '@/bindings/CreateIssuePayload'
import type { IssueAttachment } from '@/bindings/IssueAttachment'
import type { IssueAttachmentDetailDto } from '@/bindings/IssueAttachmentDetailDto'
import type { IssueAttachmentDto } from '@/bindings/IssueAttachmentDto'
import type { IssueCommentDto } from '@/bindings/IssueCommentDto'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssuePatchPayload } from '@/bindings/IssuePatchPayload'
import type { IssueStatus } from '@/bindings/IssueStatus'
import type { MyWorkResponseDto } from '@/bindings/MyWorkResponseDto'
import type { RunSummaryDto } from '@/bindings/RunSummaryDto'
import { apiClient } from './fetch'

class IssuesApi {
  public async create(data: CreateIssuePayload): Promise<IssueDto> {
    return apiClient.post('/issues', {
      body: JSON.stringify(data),
    })
  }

  public async list(): Promise<IssueDto[]> {
    return apiClient.get('/issues')
  }

  public async listByLabel(label: string | null): Promise<IssueDto[]> {
    if (!label) return this.list()
    return apiClient.get(`/issues?label=${encodeURIComponent(label)}`)
  }

  public async getMyWork(): Promise<MyWorkResponseDto> {
    return apiClient.get('/issues/my-work')
  }

  public async update(id: string, data: IssuePatchPayload): Promise<IssueDto> {
    return apiClient.patch(`/issues/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async updateStatus(id: string, status: IssueStatus): Promise<IssueDto> {
    return this.update(id, { status })
  }

  public async get(id: string): Promise<IssueDto> {
    return apiClient.get(`/issues/${id}`)
  }

  public async listRuns(id: string): Promise<RunSummaryDto[]> {
    return apiClient.get(`/issues/${id}/runs`)
  }

  public async listChildren(id: string): Promise<IssueDto[]> {
    return apiClient.get(`/issues/${id}/children`)
  }

  public async comment(id: string, data: AddCommentPayload): Promise<IssueCommentDto> {
    return apiClient.post(`/issues/${id}/comments`, {
      body: JSON.stringify(data),
    })
  }

  public async checkout(id: string): Promise<IssueDto> {
    return apiClient.post(`/issues/${id}/checkout`)
  }

  public async attachment(id: string, data: IssueAttachment): Promise<IssueAttachmentDto> {
    return apiClient.post(`/issues/${id}/attachments`, {
      body: JSON.stringify(data),
    })
  }

  public async getAttachment(id: string, attachmentId: string): Promise<IssueAttachmentDetailDto> {
    return apiClient.get(`/issues/${id}/attachments/${attachmentId}`)
  }

  public async assign(id: string, data: AssignIssuePayload): Promise<IssueDto> {
    return apiClient.post(`/issues/${id}/assign`, {
      body: JSON.stringify(data),
    })
  }

  public async unassign(id: string): Promise<IssueDto> {
    return apiClient.post(`/issues/${id}/unassign`)
  }
}

export const issuesApi = new IssuesApi()
