import type { RunStatus } from '@/bindings/RunStatus'
import { formatRunStatus, runStatusTone } from '@/lib/runs'
import { cn } from '@/lib/utils'

interface RunStatusChipProps {
  status: RunStatus
}

export const RunStatusChip = ({ status }: RunStatusChipProps) => {
  return (
    <span
      className={cn(
        'inline-flex rounded-full border border-foreground/10 bg-background px-2 py-0.5 text-[11px] font-medium capitalize',
        runStatusTone(status),
      )}
    >
      {formatRunStatus(status)}
    </span>
  )
}
