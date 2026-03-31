import type { ReactNode } from 'react'
import { BotIcon, Clock3Icon, RotateCwIcon, SquareTerminalIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { RunStatusChip } from '@/components/organisms/run-status-chip'
import { formatRunTime, formatRunTrigger } from '@/lib/runs'
import { AppModel } from '@/models/app.model'
import type { RunModel } from '@/models/run.model'
import { formatAbsoluteRunTime, getRunStats } from '../run.presenter'

interface RunHeaderProps {
  canCancel: boolean
  isCancelling: boolean
  run: RunModel
  onCancel: () => void
}

export const RunHeader = ({ canCancel, isCancelling, run, onCancel }: RunHeaderProps) => {
  const stats = getRunStats(run)
  const badges = [
    `${stats.turnCount} turn${stats.turnCount === 1 ? '' : 's'}`,
    `${stats.stepCount} step${stats.stepCount === 1 ? '' : 's'}`,
    `${stats.toolCallCount} tool call${stats.toolCallCount === 1 ? '' : 's'}`,
  ]

  return (
    <Card className="overflow-hidden border-border/60 bg-linear-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-6 md:px-6">
        <div className="flex flex-col gap-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="flex min-w-0 items-start gap-4">
              <div className="flex size-16 shrink-0 items-center justify-center rounded-2xl border border-border/60 bg-background/75 shadow-sm backdrop-blur">
                <SquareTerminalIcon className="size-7 text-muted-foreground" />
              </div>
              <div className="min-w-0 space-y-3">
                <div className="space-y-1">
                  <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Run details</p>
                  <h1 className="truncate text-2xl font-medium tracking-tight">Run {run.id.slice(0, 8)}</h1>
                  <p className="text-sm text-muted-foreground">
                    {AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'} ·{' '}
                    {formatRunTrigger(run.trigger)}
                  </p>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <RunStatusChip status={run.status} />
                  {badges.map((badge) => (
                    <span
                      key={badge}
                      className="rounded-full border border-border/60 bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground"
                    >
                      {badge}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            {canCancel ? (
              <Button disabled={isCancelling} type="button" variant="destructive-outline" onClick={onCancel}>
                {isCancelling ? 'Cancelling...' : 'Cancel run'}
              </Button>
            ) : null}
          </div>

          <div className="grid gap-3 md:grid-cols-3">
            <RunMetaStat
              icon={<Clock3Icon className="size-4" />}
              label="Created"
              value={formatAbsoluteRunTime(run.createdAt)}
              meta={formatRunTime(run.createdAt)}
            />
            <RunMetaStat
              icon={<RotateCwIcon className="size-4" />}
              label="Started"
              value={formatAbsoluteRunTime(run.startedAt)}
              meta={formatRunTime(run.startedAt)}
            />
            <RunMetaStat
              icon={<BotIcon className="size-4" />}
              label="Completed"
              value={formatAbsoluteRunTime(run.completedAt)}
              meta={run.completedAt ? formatRunTime(run.completedAt) : 'Still running'}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

const RunMetaStat = ({
  icon,
  label,
  meta,
  value,
}: {
  icon: ReactNode
  label: string
  meta: string
  value: string
}) => {
  return (
    <div className="rounded-sm border border-border/60 bg-background/75 p-4">
      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
        {icon}
        <span>{label}</span>
      </div>
      <p className="mt-3 text-sm font-medium">{value}</p>
      <p className="mt-1 text-xs text-muted-foreground">{meta}</p>
    </div>
  )
}
