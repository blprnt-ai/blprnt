import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { Provider } from '@/bindings/Provider'

export const formatLabel = (value: string) => {
  return value
    .split(/[_-]/g)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export const formatRole = (role: EmployeeRole) => {
  if (typeof role === 'string') return formatLabel(role)
  if ('custom' in role) return role.custom

  return 'Unknown'
}

export const formatProvider = (provider: Provider) => {
  switch (provider) {
    case 'claude_code':
      return 'Claude Code'
    case 'open_router':
      return 'OpenRouter'
    default:
      return formatLabel(provider)
  }
}

export const isSameProvider = (provider: Provider, otherProvider: Provider) => {
  return (
    provider === otherProvider ||
    (isOpenAi(provider) && isOpenAi(otherProvider)) ||
    (isAnthropic(provider) && isAnthropic(otherProvider))
  )
}

export const isOpenAi = (provider: Provider) => provider === 'openai' || provider === 'codex'
export const isAnthropic = (provider: Provider) => provider === 'anthropic' || provider === 'claude_code'

export const formatCapabilities = (capabilities: string[]) => {
  if (capabilities.length === 0) return 'No capabilities listed'

  return capabilities.join(', ')
}
