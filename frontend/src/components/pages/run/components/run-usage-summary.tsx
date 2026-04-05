import { CoinsIcon } from 'lucide-react'
import { getUsageSummary } from '@/lib/usage'
import { cn } from '@/lib/utils'
import type { UsageMetricsModel } from '@/models/usage-metrics.model'

interface RunUsageSummaryProps {
  usage: UsageMetricsModel
  compact?: boolean
  className?: string
}

export const RunUsageSummary = ({ usage, compact = false, className }: RunUsageSummaryProps) => {
  const summary = getUsageSummary(usage)
  const notes = [summary.hasUnavailableTokenData ? '' : null, summary.hasUnavailableCostData ? '' : null].filter(
    Boolean,
  )

  if (compact) {
    return (
      <div className={cn('flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground', className)}>
        {summary.cost ? <UsageMetric label="Cost" value={summary.cost} /> : null}
        {summary.totalTokens ? <UsageMetric label="Tokens" value={summary.totalTokens} /> : null}
        {summary.source ? <span>{summary.source}</span> : null}
        {notes.map((note) => (
          <span key={note} className="text-amber-600">
            {note}
          </span>
        ))}
      </div>
    )
  }

  return (
    <section className={cn('rounded-xl border border-border/60 bg-background/60 p-4', className)}>
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
            <CoinsIcon className="size-3.5" />
            Usage
          </div>
          {summary.source ? <p className="text-sm text-muted-foreground">{summary.source}</p> : null}
        </div>
        <div className="flex flex-wrap gap-2">
          {summary.cost ? <UsageBadge label="Cost" value={summary.cost} /> : null}
          {summary.totalTokens ? <UsageBadge label="Total tokens" value={summary.totalTokens} /> : null}
          {summary.inputTokens ? <UsageBadge label="Input" value={summary.inputTokens} /> : null}
          {summary.outputTokens ? <UsageBadge label="Output" value={summary.outputTokens} /> : null}
        </div>
      </div>
      {notes.length > 0 ? (
        <div className="mt-3 flex flex-wrap gap-2 text-xs text-amber-600">
          {notes.map((note) => (
            <span key={note}>{note}</span>
          ))}
        </div>
      ) : null}
      {!summary.hasAnyMetric && notes.length === 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">Usage data has not been recorded for this item yet.</p>
      ) : null}
    </section>
  )
}

const UsageBadge = ({ label, value }: { label: string; value: string }) => {
  return (
    <div className="min-w-24 rounded-lg border border-border/60 bg-background px-3 py-2">
      <p className="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">{label}</p>
      <p className="mt-1 text-sm font-medium text-foreground">{value}</p>
    </div>
  )
}

const UsageMetric = ({ label, value }: { label: string; value: string }) => {
  return (
    <span>
      <span className="uppercase tracking-[0.18em] text-muted-foreground">{label}</span> {value}
    </span>
  )
}
