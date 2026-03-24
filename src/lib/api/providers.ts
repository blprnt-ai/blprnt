import type { CreateProviderPayload } from '@/bindings/CreateProviderPayload'
import type { ProviderDto } from '@/bindings/ProviderDto'
import type { ProviderPatch } from '@/bindings/ProviderPatch'
import { apiClient } from './fetch'

class ProvidersApi {
  public async list(): Promise<ProviderDto[]> {
    return apiClient.get('/providers')
  }

  public async create(data: CreateProviderPayload): Promise<ProviderDto> {
    return apiClient.post('/providers', {
      body: JSON.stringify(data),
    })
  }

  public async get(id: string): Promise<ProviderDto> {
    return apiClient.get(`/providers/${id}`)
  }

  public async update(id: string, data: ProviderPatch): Promise<ProviderDto> {
    return apiClient.patch(`/providers/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async delete(id: string): Promise<void> {
    return apiClient.delete(`/providers/${id}`)
  }
}

export const providersApi = new ProvidersApi()
