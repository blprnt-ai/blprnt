import type { AppendRunMessagePayload } from '@/bindings/AppendRunMessagePayload'
import type { RunDto } from '@/bindings/RunDto'
import type { RunStatus } from '@/bindings/RunStatus'
import type { RunSummaryDto } from '@/bindings/RunSummaryDto'
import type { RunSummaryPageDto } from '@/bindings/RunSummaryPageDto'
import type { TriggerRunPayload } from '@/bindings/TriggerRunPayload'
import { apiClient } from './fetch'

export type RunStatusFilter = Extract<RunStatus, string> | 'Failed'

class RunsApi {
  public async list(
    page = 1,
    perPage = 20,
    options?: { employeeId?: string | null, status?: RunStatusFilter | null },
  ): Promise<RunSummaryPageDto> {
    const search = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
    })

    if (options?.employeeId) {
      search.set('employee', options.employeeId)
    }

    if (options?.status === 'Failed') {
      search.set('status.Failed', '')
    } else if (options?.status) {
      search.set('status', options.status)
    }

    return apiClient.get(`/runs?${search.toString()}`)
  }

  public async listForEmployee(employeeId: string, perPage = 100): Promise<RunSummaryDto[]> {
    const response = await this.list(1, perPage, { employeeId })

    return response.items.sort(
      (left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime(),
    )
  }

  public async get(id: string): Promise<RunDto> {
    return apiClient.get(`/runs/${id}`)
  }

  public async trigger(data: TriggerRunPayload): Promise<RunDto> {
    return apiClient.post('/runs', {
      body: JSON.stringify(data),
    })
  }

  public async appendMessage(id: string, data: AppendRunMessagePayload): Promise<RunDto> {
    return apiClient.post(`/runs/${id}/messages`, {
      body: JSON.stringify(data),
    })
  }

  public async cancel(id: string): Promise<void> {
    return apiClient.delete(`/runs/${id}/cancel`)
  }
}

export const runsApi = new RunsApi()
