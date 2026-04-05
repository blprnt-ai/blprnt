import type { Provider } from '@/bindings/Provider'

export type SupportedProvider = Exclude<Provider, 'mock'>

export interface ProviderOption {
  description: string
  provider: SupportedProvider
  title: string
}

export const PROVIDER_OPTIONS: ProviderOption[] = [
  {
    description: 'Use your Claude Code subscription.',
    provider: 'claude_code',
    title: 'Claude Code',
  },
  {
    description: 'Use your Codex subscription.',
    provider: 'codex',
    title: 'Codex',
  },
  {
    description: 'Use your OpenRouter API key.',
    provider: 'open_router',
    title: 'OpenRouter',
  },
  {
    description: 'Use your Anthropic API key.',
    provider: 'anthropic',
    title: 'Anthropic',
  },
  {
    description: 'Use your OpenAI API key.',
    provider: 'openai',
    title: 'OpenAI',
  },
]

export const getProviderOption = (provider: Provider) => {
  return PROVIDER_OPTIONS.find((option) => option.provider === provider) ?? null
}

export const isOauthProvider = (provider: Provider) => provider === 'claude_code' || provider === 'codex'
