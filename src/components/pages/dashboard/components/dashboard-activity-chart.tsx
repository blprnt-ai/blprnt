import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

interface DashboardActivityChartProps {
  points: Array<{ label: string; runCount: number; completedCount: number }>
}

export const DashboardActivityChart = ({ points }: DashboardActivityChartProps) => {
  const maxValue = Math.max(...points.flatMap((point) => [point.runCount, point.completedCount]), 1)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Run activity</CardTitle>
        <CardDescription>Created vs completed runs over the last 7 days.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-7 items-end gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
          {points.map((point) => (
            <div key={point.label} className="flex min-h-52 flex-col justify-end gap-3">
              <div className="flex flex-1 items-end justify-center gap-1.5">
                <div
                  className="w-3 rounded-full bg-chart-1/90"
                  style={{ height: `${Math.max((point.runCount / maxValue) * 160, point.runCount > 0 ? 10 : 2)}px` }}
                />
                <div
                  className="w-3 rounded-full bg-chart-3/90"
                  style={{ height: `${Math.max((point.completedCount / maxValue) * 160, point.completedCount > 0 ? 10 : 2)}px` }}
                />
              </div>
              <div className="space-y-1 text-center">
                <p className="text-xs font-medium text-foreground">{point.label}</p>
                <p className="text-[11px] text-muted-foreground">{point.runCount} / {point.completedCount}</p>
              </div>
            </div>
          ))}
        </div>
        <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
          <LegendDot className="bg-chart-1" label="Created" />
          <LegendDot className="bg-chart-3" label="Completed" />
        </div>
      </CardContent>
    </Card>
  )
}

const LegendDot = ({ className, label }: { className: string; label: string }) => {
  return (
    <div className="flex items-center gap-2">
      <span className={cn('size-2.5 rounded-full', className)} />
      <span>{label}</span>
    </div>
  )
}