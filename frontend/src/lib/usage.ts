import type { UsageMetricsModel } from '@/models/usage-metrics.model'

const tokenFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 })

export const formatTokenCount = (value: number | bigint | null) => {
  if (value === null) return null
  return tokenFormatter.format(value)
}

export const formatUsageCost = (value: number | null) => {
  if (value === null) return null

  return new Intl.NumberFormat(undefined, {
    currency: 'USD',
    maximumFractionDigits: value < 0.01 ? 4 : 2,
    minimumFractionDigits: value < 0.01 ? 4 : 2,
    style: 'currency',
  }).format(value)
}

export const formatUsageSource = (usage: UsageMetricsModel) => {
  if (usage.provider && usage.model) return `${formatProvider(usage.provider)} · ${usage.model}`
  if (usage.provider) return formatProvider(usage.provider)
  if (usage.model) return usage.model
  return null
}

export const getUsageSummary = (usage: UsageMetricsModel) => {
  const cost = formatUsageCost(usage.estimatedCostUsd)
  const inputTokens = formatTokenCount(usage.inputTokens)
  const outputTokens = formatTokenCount(usage.outputTokens)
  const totalTokens = formatTokenCount(usage.totalTokens)
  const hasAnyMetric = Boolean(cost || inputTokens || outputTokens || totalTokens)

  return {
    cost,
    hasAnyMetric,
    hasUnavailableCostData: usage.hasUnavailableCostData,
    hasUnavailableTokenData: usage.hasUnavailableTokenData,
    inputTokens,
    outputTokens,
    source: formatUsageSource(usage),
    totalTokens,
  }
}

const formatProvider = (provider: string) => provider.replace(/_/g, ' ')
