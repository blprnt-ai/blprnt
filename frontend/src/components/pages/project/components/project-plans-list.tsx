import { FileText } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { useProjectViewmodel } from '../project.viewmodel'
import { formatRelativeDateTime } from '../utils'

export const ProjectPlansList = observer(() => {
  const viewmodel = useProjectViewmodel()

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle className="text-base">Plans</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        {viewmodel.plans.map((plan) => {
          const isSelected = viewmodel.selectedPlanPath === plan.path

          return (
            <button
              key={plan.path}
              className={cn(
                'w-full rounded-md border border-border/60 px-3 py-3 text-left transition hover:bg-muted/30',
                isSelected && 'border-primary/30 bg-muted/40',
              )}
              type="button"
              onClick={() => void viewmodel.selectPlanPath(plan.path)}
            >
              <div className="flex items-start gap-3">
                <FileText className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                <div className="min-w-0 flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="truncate text-sm font-medium">{plan.title || plan.filename}</div>
                    {plan.is_superseded ? (
                      <span className="shrink-0 rounded-full border border-border/70 px-2 py-0.5 text-[11px] text-muted-foreground">
                        Superseded
                      </span>
                    ) : null}
                  </div>
                  <div className="truncate text-xs text-muted-foreground">{plan.filename}</div>
                  <div className="text-xs text-muted-foreground">Updated {formatRelativeDateTime(plan.updated_at)}</div>
                </div>
              </div>
            </button>
          )
        })}
      </CardContent>
    </Card>
  )
})
