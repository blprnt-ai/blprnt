import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { CreateTelegramLinkCodeResponse } from '@/bindings/CreateTelegramLinkCodeResponse'
import type { TelegramDeliveryMode } from '@/bindings/TelegramDeliveryMode'
import type { TelegramLinkDto } from '@/bindings/TelegramLinkDto'
import type { TelegramParseMode } from '@/bindings/TelegramParseMode'
import type { UpsertTelegramConfigPayload } from '@/bindings/UpsertTelegramConfigPayload'
import { telegramApi } from '@/lib/api/telegram'

export class TelegramViewmodel {
  public ownerId: string
  public isLoading = true
  public isSaving = false
  public isGeneratingCode = false
  public errorMessage: string | null = null
  public saveMessage: string | null = null
  public botToken = ''
  public webhookSecret = ''
  public botUsername = ''
  public webhookUrl = ''
  public enabled = false
  public deliveryMode: TelegramDeliveryMode = 'webhook'
  public parseMode: TelegramParseMode | null = null
  public links: TelegramLinkDto[] = []
  public latestLinkCode: CreateTelegramLinkCodeResponse | null = null

  constructor(ownerId: string) {
    this.ownerId = ownerId
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const [config, links] = await Promise.all([telegramApi.getConfig(), telegramApi.listLinks(this.ownerId)])

      runInAction(() => {
        this.links = links
        this.enabled = config?.enabled ?? false
        this.botUsername = config?.bot_username ?? ''
        this.webhookUrl = config?.webhook_url ?? ''
        this.deliveryMode = config?.delivery_mode ?? 'webhook'
        this.parseMode = config?.parse_mode ?? null
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load Telegram settings.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public setEnabled(value: boolean) {
    this.enabled = value
  }

  public setBotToken(value: string) {
    this.botToken = value
  }

  public setBotUsername(value: string) {
    this.botUsername = value
  }

  public setDeliveryMode(value: TelegramDeliveryMode) {
    this.deliveryMode = value
  }

  public setParseMode(value: TelegramParseMode | null) {
    this.parseMode = value
  }

  public setWebhookUrl(value: string) {
    this.webhookUrl = value
  }

  public setWebhookSecret(value: string) {
    this.webhookSecret = value
  }

  public async saveConfig() {
    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
      this.saveMessage = null
    })

    const payload: UpsertTelegramConfigPayload = {
      bot_token: this.botToken,
      bot_username: emptyToNull(this.botUsername),
      delivery_mode: this.deliveryMode,
      enabled: this.enabled,
      parse_mode: this.parseMode,
      webhook_secret: this.webhookSecret,
      webhook_url: emptyToNull(this.webhookUrl),
    }

    try {
      const config = await telegramApi.saveConfig(payload)
      runInAction(() => {
        this.enabled = config.enabled
        this.botUsername = config.bot_username ?? ''
        this.webhookUrl = config.webhook_url ?? ''
        this.deliveryMode = config.delivery_mode
        this.parseMode = config.parse_mode ?? null
        this.botToken = ''
        this.webhookSecret = ''
        this.saveMessage = 'Telegram settings saved.'
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save Telegram settings.')
      })
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  public async generateLinkCode() {
    runInAction(() => {
      this.isGeneratingCode = true
      this.errorMessage = null
    })

    try {
      const response = await telegramApi.createLinkCode({ employee_id: this.ownerId })
      const links = await telegramApi.listLinks(this.ownerId)

      runInAction(() => {
        this.latestLinkCode = response
        this.links = links
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to generate a Telegram link code.')
      })
    } finally {
      runInAction(() => {
        this.isGeneratingCode = false
      })
    }
  }

  public async refreshLinks() {
    try {
      const links = await telegramApi.listLinks(this.ownerId)
      runInAction(() => {
        this.links = links
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to refresh linked chats.')
      })
    }
  }

  public get botHandle() {
    return this.botUsername.trim().replace(/^@/, '')
  }

  public get linkCommand() {
    return this.latestLinkCode ? `/link ${this.latestLinkCode.code}` : '/link <code>'
  }
}

export const TelegramViewmodelContext = createContext<TelegramViewmodel | null>(null)

export const useTelegramViewmodel = () => {
  const viewmodel = useContext(TelegramViewmodelContext)
  if (!viewmodel) throw new Error('TelegramViewmodel not found')

  return viewmodel
}

const emptyToNull = (value: string) => {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}
