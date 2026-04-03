import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { PROVIDER_OPTIONS } from '@/components/forms/provider/provider-catalog'
import type { LabeledSelectOption } from '@/components/molecules/labeled-select'

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

export const getRuntimeProviderOptions = ({
  configuredProviders,
  currentProvider,
  disableUnconfiguredProviders,
}: {
  configuredProviders: ProviderDto[]
  currentProvider: Provider
  disableUnconfiguredProviders: boolean
}): Array<LabeledSelectOption & { value: Provider }> => {
  const configuredProviderSet = new Set(configuredProviders.map((provider) => provider.provider))

  return PROVIDER_OPTIONS.map((option) => ({
    disabled:
      disableUnconfiguredProviders &&
      option.provider !== currentProvider &&
      !configuredProviderSet.has(option.provider),
    label: option.title,
    value: option.provider,
  }))
}

export const formatCapabilities = (capabilities: string[]) => {
  if (capabilities.length === 0) return 'No capabilities listed'

  return capabilities.join(', ')
}

export const canReportTo = (employeeRole: EmployeeRole, managerRole: EmployeeRole) => {
  if (typeof employeeRole !== 'string' || typeof managerRole !== 'string') return false
  if (employeeRole === 'owner') return false
  if (managerRole === 'owner') return true
  if (employeeRole === 'ceo') return false
  if (employeeRole === 'manager') return managerRole === 'ceo'

  return managerRole === 'ceo' || managerRole === 'manager'
}
