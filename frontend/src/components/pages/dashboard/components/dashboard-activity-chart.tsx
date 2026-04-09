import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

interface DashboardActivityChartProps {
  points: Array<{
    label: string
    criticalCount: number
    highCount: number
    mediumCount: number
    lowCount: number
  }>
}

export const DashboardActivityChart = ({ points }: DashboardActivityChartProps) => {
  const maxValue = Math.max(
    ...points.flatMap((point) => [point.criticalCount, point.highCount, point.mediumCount, point.lowCount]),
    1,
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Run activity</CardTitle>
        <CardDescription>Completed issues over the last 7 days, broken down by priority.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-7 items-end gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
          {points.map((point) => (
            <div key={point.label} className="flex min-h-52 flex-col justify-end gap-3">
              <div className="flex flex-1 items-end justify-center gap-1.5">
                <div
                  className="w-3 rounded-full bg-chart-5/90"
                  style={{
                    height: `${Math.max((point.criticalCount / maxValue) * 160, point.criticalCount > 0 ? 10 : 2)}px`,
                  }}
                />
                <div
                  className="w-3 rounded-full bg-chart-4/90"
                  style={{ height: `${Math.max((point.highCount / maxValue) * 160, point.highCount > 0 ? 10 : 2)}px` }}
                />
                <div
                  className="w-3 rounded-full bg-chart-2/90"
                  style={{
                    height: `${Math.max((point.mediumCount / maxValue) * 160, point.mediumCount > 0 ? 10 : 2)}px`,
                  }}
                />
                <div
                  className="w-3 rounded-full bg-chart-1/90"
                  style={{ height: `${Math.max((point.lowCount / maxValue) * 160, point.lowCount > 0 ? 10 : 2)}px` }}
                />
              </div>
              <div className="space-y-1 text-center">
                <p className="text-xs font-medium text-foreground">{point.label}</p>
                <p className="text-[11px] text-muted-foreground">
                  {point.criticalCount + point.highCount + point.mediumCount + point.lowCount} completed
                </p>
              </div>
            </div>
          ))}
        </div>
        <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
          <LegendDot className="bg-chart-5" label="Critical" />
          <LegendDot className="bg-chart-4" label="High" />
          <LegendDot className="bg-chart-2" label="Medium" />
          <LegendDot className="bg-chart-1" label="Low" />
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
