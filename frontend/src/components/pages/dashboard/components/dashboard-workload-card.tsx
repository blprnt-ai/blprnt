import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

interface BreakdownItem {
  label: string
  value: number
  tone: string
}

interface DashboardWorkloadCardProps {
  items: BreakdownItem[]
  priorityItems: BreakdownItem[]
}

export const DashboardWorkloadCard = ({ items, priorityItems }: DashboardWorkloadCardProps) => {
  const total = Math.max(items.reduce((sum, item) => sum + item.value, 0), 1)
  const priorityTotal = Math.max(priorityItems.reduce((sum, item) => sum + item.value, 0), 1)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Workload mix</CardTitle>
        <CardDescription>What is active, what is blocked, and where urgency is accumulating.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <BreakdownSection items={items} title="Issue status" total={total} />
        <BreakdownSection items={priorityItems} title="Open priority" total={priorityTotal} />
      </CardContent>
    </Card>
  )
}

const BreakdownSection = ({ items, title, total }: { items: BreakdownItem[]; title: string; total: number }) => {
  return (
    <div className="space-y-3">
      <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">{title}</p>
      <div className="space-y-3">
        {items.map((item) => (
          <div key={item.label} className="space-y-1.5">
            <div className="flex items-center justify-between gap-3 text-sm">
              <div className="flex items-center gap-2">
                <span className={cn('size-2.5 rounded-full', item.tone)} />
                <span>{item.label}</span>
              </div>
              <span className="text-muted-foreground">{item.value}</span>
            </div>
            <div className="h-2 rounded-full bg-muted">
              <div className={cn('h-2 rounded-full', item.tone)} style={{ width: `${(item.value / total) * 100}%` }} />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}