import type { RunDto } from '@/bindings/RunDto'
import type { TriggerRunPayload } from '@/bindings/TriggerRunPayload'
import { apiClient } from './fetch'

class RunsApi {
  public async list(): Promise<RunDto[]> {
    return apiClient.get('/runs')
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
