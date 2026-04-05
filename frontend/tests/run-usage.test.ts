import { describe, expect, it } from 'vitest'
import { getUsageSummary } from '@/lib/usage'
import { UsageMetricsModel } from '@/models/usage-metrics.model'

const usage = (overrides: Partial<ConstructorParameters<typeof UsageMetricsModel>[0]> = {}) =>
  new UsageMetricsModel({
    provider: null,
    model: null,
    input_tokens: null,
    output_tokens: null,
    total_tokens: null,
    estimated_cost_usd: null,
    has_unavailable_token_data: false,
    has_unavailable_cost_data: false,
    ...overrides,
  })

describe('run usage formatting', () => {
  it('formats token and cost values for display', () => {
    const summary = getUsageSummary(
      usage({
        provider: 'openai',
        model: 'gpt-5',
        input_tokens: 1200n,
        output_tokens: 300n,
        total_tokens: 1500n,
        estimated_cost_usd: 0.0375,
      }),
    )

    expect(summary.source).toBe('openai · gpt-5')
    expect(summary.inputTokens).toBe('1,200')
    expect(summary.outputTokens).toBe('300')
    expect(summary.totalTokens).toBe('1,500')
    expect(summary.cost).toBe('$0.04')
    expect(summary.hasAnyMetric).toBe(true)
  })

  it('preserves unavailable flags when values are missing', () => {
    const summary = getUsageSummary(
      usage({
        has_unavailable_token_data: true,
        has_unavailable_cost_data: true,
      }),
    )

    expect(summary.cost).toBeNull()
    expect(summary.totalTokens).toBeNull()
    expect(summary.hasUnavailableTokenData).toBe(true)
    expect(summary.hasUnavailableCostData).toBe(true)
    expect(summary.hasAnyMetric).toBe(false)
  })
})