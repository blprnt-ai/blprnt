import type { RunDto } from '@/bindings/RunDto'
import type { RunSummaryPageDto } from '@/bindings/RunSummaryPageDto'
import type { TriggerRunPayload } from '@/bindings/TriggerRunPayload'
import { apiClient } from './fetch'

class RunsApi {
  public async list(page = 1, perPage = 20): Promise<RunSummaryPageDto> {
    const search = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
    })

    return apiClient.get(`/runs?${search.toString()}`)
  }

  public async get(id: string): Promise<RunDto> {
    return apiClient.get(`/runs/${id}`)
  }

  public async trigger(data: TriggerRunPayload): Promise<RunDto> {
    return apiClient.post('/runs', {
      body: JSON.stringify(data),
    })
  }

  public async cancel(id: string): Promise<void> {
    return apiClient.delete(`/runs/${id}/cancel`)
  }
}

export const runsApi = new RunsApi()
