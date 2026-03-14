import type { AllProviders } from '@/types'

class ApiKeyValidator {
  public isValid(provider: AllProviders, key: string): boolean {
    switch (provider) {
      case 'openai':
        return this.isValidOpenAIKey(key)
      case 'anthropic':
        return this.isValidAnthropicKey(key)
      default:
        return false
    }
  }

  private isValidOpenAIKey(key: string): boolean {
    if (!key) return false

    return /^sk-(proj|svcacct)-[A-Za-z0-9\-_]{156}$/.test(key.trim())
  }

  private isValidAnthropicKey(key: string): boolean {
    if (!key) return false

    return /^sk-ant-[A-Za-z0-9\-_]{101}$/.test(key.trim())
  }
}

export const apiKeyValidator = new ApiKeyValidator()
