import type { CreateMinionPayload } from '@/bindings/CreateMinionPayload'
import type { MinionDto } from '@/bindings/MinionDto'
import type { MinionPatchPayload } from '@/bindings/MinionPatchPayload'
import { apiClient } from './fetch'

class MinionsApi {
  public async list(): Promise<MinionDto[]> {
    return apiClient.get('/minions')
  }

  public async create(data: CreateMinionPayload): Promise<MinionDto> {
    return apiClient.post('/minions', {
      body: JSON.stringify(data),
    })
  }

  public async update(id: string, data: MinionPatchPayload): Promise<MinionDto> {
    return apiClient.patch(`/minions/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async delete(id: string): Promise<void> {
    return apiClient.delete(`/minions/${id}`)
  }
}

export const minionsApi = new MinionsApi()