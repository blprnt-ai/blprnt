import { Clock3Icon, RotateCwIcon, SquareTerminalIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { formatRunStatus, formatRunTrigger, formatRunTime, runStatusTone } from '@/lib/runs'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import type { RunModel } from '@/models/run.model'
import { formatAbsoluteRunTime, getRunStats } from '../run.presenter'

interface RunHeaderProps {
  canCancel: boolean
  isCancelling: boolean
  run: RunModel
  onCancel: () => void
}

export const RunHeader = observer(({ canCancel, isCancelling, run, onCancel }: RunHeaderProps) => {
  const stats = getRunStats(run)
  const badges = [
    `${stats.turnCount} turn${stats.turnCount === 1 ? '' : 's'}`,
    `${stats.stepCount} step${stats.stepCount === 1 ? '' : 's'}`,
    `${stats.toolCallCount} tool call${stats.toolCallCount === 1 ? '' : 's'}`,
  ]
  const title = run.trigger === 'conversation' ? 'Conversation' : `Run ${run.id.slice(0, 8)}`

  return (
    <Card className="overflow-hidden border-border/60 bg-linear-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-5 md:px-6">
        <div className="flex flex-col gap-4">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="flex min-w-0 items-start gap-4">
              <div className="flex size-11 shrink-0 items-center justify-center rounded-xl border border-border/60 bg-background/75">
                <SquareTerminalIcon className="size-5 text-muted-foreground" />
              </div>
              <div className="min-w-0 space-y-2">
                <div className="space-y-1">
                  <h1 className="truncate text-xl font-medium tracking-tight">{title}</h1>
                  <p className="text-sm text-muted-foreground">
                    {AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'} ·{' '}
                    {formatRunTrigger(run.trigger)}
                  </p>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <span
                    className={cn(
                      'rounded-full border border-border/60 bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em]',
                      runStatusTone(run.status),
                    )}
                  >
                    {formatRunStatus(run.status)}
                  </span>
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

          <div className="flex flex-wrap gap-x-5 gap-y-2 text-sm text-muted-foreground">
            <RunMetaInline
              icon={<Clock3Icon className="size-4" />}
              label="Created"
              value={formatAbsoluteRunTime(run.createdAt)}
            />
            <RunMetaInline
              icon={<RotateCwIcon className="size-4" />}
              label="Started"
              value={formatAbsoluteRunTime(run.startedAt)}
            />
            <RunMetaInline
              label="Completed"
              value={run.completedAt ? formatAbsoluteRunTime(run.completedAt) : 'Still running'}
            />
            <RunMetaInline
              label="Last activity"
              value={run.completedAt ? formatRunTime(run.completedAt) : formatRunTime(run.startedAt)}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  )
})

const RunMetaInline = ({ icon, label, value }: { icon?: React.ReactNode; label: string; value: string }) => {
  return (
    <div className="flex items-center gap-2">
      {icon ? <span className="text-muted-foreground">{icon}</span> : null}
      <span className="text-xs uppercase tracking-[0.18em] text-muted-foreground">{label}</span>
      <span className="font-medium text-foreground">{value}</span>
    </div>
  )
}
