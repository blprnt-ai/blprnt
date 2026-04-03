import type { CreateTelegramLinkCodePayload } from '@/bindings/CreateTelegramLinkCodePayload'
import type { CreateTelegramLinkCodeResponse } from '@/bindings/CreateTelegramLinkCodeResponse'
import type { TelegramConfigDto } from '@/bindings/TelegramConfigDto'
import type { TelegramLinkDto } from '@/bindings/TelegramLinkDto'
import type { UpsertTelegramConfigPayload } from '@/bindings/UpsertTelegramConfigPayload'
import { apiClient } from './fetch'

class TelegramApi {
  public async getConfig(): Promise<TelegramConfigDto | null> {
    return apiClient.get('/integrations/telegram/config')
  }

  public async saveConfig(data: UpsertTelegramConfigPayload): Promise<TelegramConfigDto> {
    return apiClient.post('/integrations/telegram/config', {
      body: JSON.stringify(data),
    })
  }

  public async createLinkCode(data: CreateTelegramLinkCodePayload): Promise<CreateTelegramLinkCodeResponse> {
    return apiClient.post('/integrations/telegram/link-codes', {
      body: JSON.stringify(data),
    })
  }

  public async listLinks(employeeId: string): Promise<TelegramLinkDto[]> {
    return apiClient.get(`/integrations/telegram/links/${employeeId}`)
  }
}

export const telegramApi = new TelegramApi()