import type { UsageMetrics } from '@/bindings/UsageMetrics'

export class UsageMetricsModel {
  public provider: string | null
  public model: string | null
  public inputTokens: number | null
  public outputTokens: number | null
  public totalTokens: number | null
  public estimatedCostUsd: number | null
  public hasUnavailableTokenData: boolean
  public hasUnavailableCostData: boolean

  constructor(usage: UsageMetrics | null) {
    this.provider = usage?.provider ? String(usage.provider) : null
    this.model = usage?.model ?? null
    this.inputTokens = toNullableNumber(usage?.input_tokens ?? null)
    this.outputTokens = toNullableNumber(usage?.output_tokens ?? null)
    this.totalTokens = toNullableNumber(usage?.total_tokens ?? null)
    this.estimatedCostUsd = usage?.estimated_cost_usd ?? null
    this.hasUnavailableTokenData = usage?.has_unavailable_token_data ?? false
    this.hasUnavailableCostData = usage?.has_unavailable_cost_data ?? false
  }
}

const toNullableNumber = (value: bigint | number | null) => {
  if (value === null || value === undefined) return null
  return Number(value)
}
