import type { AddCommentPayload } from '@/bindings/AddCommentPayload'
import type { AssignIssuePayload } from '@/bindings/AssignIssuePayload'
import type { CreateIssuePayload } from '@/bindings/CreateIssuePayload'
import type { IssueAttachment } from '@/bindings/IssueAttachment'
import type { IssueAttachmentDto } from '@/bindings/IssueAttachmentDto'
import type { IssueCommentDto } from '@/bindings/IssueCommentDto'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssuePatch } from '@/bindings/IssuePatch'
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

  public async update(id: string, data: IssuePatch): Promise<IssueDto> {
    return apiClient.patch(`/issues/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async get(id: string): Promise<IssueDto> {
    return apiClient.get(`/issues/${id}`)
  }

  public async comment(id: string, data: AddCommentPayload): Promise<IssueCommentDto> {
    return apiClient.post(`/issues/${id}/comments`, {
      body: JSON.stringify(data),
    })
  }

  public async attachment(id: string, data: IssueAttachment): Promise<IssueAttachmentDto> {
    return apiClient.post(`/issues/${id}/attachments`, {
      body: JSON.stringify(data),
    })
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
