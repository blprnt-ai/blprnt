import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

interface DashboardProjectHealthProps {
  items: Array<{ id: string; name: string; totalIssues: number; openIssues: number; completedIssues: number; runCount: number }>
}

export const DashboardProjectHealth = ({ items }: DashboardProjectHealthProps) => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Project health</CardTitle>
        <CardDescription>Projects with the most visible motion and remaining pressure.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {items.length === 0 ? <p className="text-sm text-muted-foreground">Projects will appear here once work is linked to them.</p> : null}
        {items.map((item) => {
          const completion = item.totalIssues === 0 ? 0 : Math.round((item.completedIssues / item.totalIssues) * 100)

          return (
            <div key={item.id} className="rounded-2xl border border-border/60 bg-muted/20 p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="font-medium text-foreground">{item.name}</p>
                  <p className="text-sm text-muted-foreground">{item.openIssues} open · {item.runCount} linked runs</p>
                </div>
                <p className="text-sm font-medium text-foreground">{completion}%</p>
              </div>
              <div className="mt-3 h-2 rounded-full bg-muted">
                <div className="h-2 rounded-full bg-chart-1" style={{ width: `${completion}%` }} />
              </div>
            </div>
          )
        })}
      </CardContent>
    </Card>
  )
}