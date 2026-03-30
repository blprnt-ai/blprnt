import { PauseIcon, PenLineIcon, PlayIcon, Trash2Icon } from 'lucide-react'
import { Identity } from '@/components/molecules/indentity'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'
import { formatLabel, formatProvider, formatRole } from '../utils'

export const EmployeeHeader = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const statusItems = [
    formatRole(employee.role),
    formatLabel(employee.kind),
    viewmodel.showsAgentConfiguration ? formatLabel(employee.status) : null,
    viewmodel.showsAgentConfiguration ? formatProvider(employee.provider) : null,
  ].filter(Boolean)

  return (
    <Card className="overflow-hidden border-border/60 bg-linear-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-6 md:px-6">
        <div className="space-y-4">
          <div className="flex flex-wrap items-start gap-4">
            <div className="rounded-2xl border border-border/60 bg-background/75 p-4 shadow-sm backdrop-blur">
              <Identity className="text-lg" color={employee.color} icon={employee.icon} size="lg" />
            </div>
            <div className="min-w-0 flex-1 space-y-2">
              <div>
                <h2 className="truncate text-2xl font-medium tracking-tight">{employee.name}</h2>
              </div>
              <div className="flex flex-wrap gap-2">
                {statusItems.map((item) => (
                  <span
                    key={item}
                    className="rounded-full border border-border/60 bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground"
                  >
                    {item}
                  </span>
                ))}
              </div>
            </div>
            {viewmodel.showsAgentConfiguration ? (
              <div className="flex w-full flex-wrap gap-2 md:w-auto md:justify-end">
                <Button
                  disabled={viewmodel.isTerminated || viewmodel.isStatusUpdating || viewmodel.isTerminating}
                  type="button"
                  variant="outline"
                  onClick={() => void viewmodel.togglePaused()}
                >
                  {viewmodel.isPaused ? <PlayIcon /> : <PauseIcon />}
                  {viewmodel.isStatusUpdating ? viewmodel.pauseResumePendingLabel : viewmodel.pauseResumeLabel}
                </Button>
                <Button
                  disabled={viewmodel.isTerminated || viewmodel.isTerminating}
                  type="button"
                  variant="secondary"
                  onClick={viewmodel.openAddIssue}
                >
                  <PenLineIcon />
                  Add issue
                </Button>
                <Button
                  disabled={viewmodel.isTerminating}
                  type="button"
                  variant="destructive-outline"
                  onClick={() => {
                    if (!window.confirm(`Terminate ${employee.name}? This cannot be undone.`)) return
                    void viewmodel.terminate()
                  }}
                >
                  <Trash2Icon />
                  {viewmodel.isTerminating ? 'Terminating...' : 'Terminate'}
                </Button>
              </div>
            ) : null}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
