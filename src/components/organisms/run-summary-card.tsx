import { Link } from '@tanstack/react-router'
import { ChevronRightIcon } from 'lucide-react'
import type { RunSummaryModel } from '@/models/run-summary.model'
import { AppModel } from '@/models/app.model'
import { formatRunTime, formatRunTrigger } from '@/lib/runs'
import { RunStatusChip } from './run-status-chip'

interface RunSummaryCardProps {
  run: RunSummaryModel
  latestActivity?: string | null
}

export const RunSummaryCard = ({ run, latestActivity }: RunSummaryCardProps) => {
  return (
    <Link
      params={{ runId: run.id }}
      to="/runs/$runId"
      className="group flex items-start justify-between gap-4 rounded-sm border border-foreground/10 bg-card px-4 py-3 transition-colors hover:bg-muted/50"
    >
      <div className="min-w-0 flex-1 space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <p className="font-medium">{AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'}</p>
          <RunStatusChip status={run.status} />
        </div>
        <p className="text-xs text-muted-foreground">
          {formatRunTrigger(run.trigger)} · {formatRunTime(run.startedAt ?? run.createdAt)}
        </p>
        <p className="truncate text-sm text-muted-foreground">{latestActivity?.trim() || 'Open run details'}</p>
      </div>
      <ChevronRightIcon className="mt-0.5 size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
    </Link>
  )
}
