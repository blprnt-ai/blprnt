import { Identity } from '@/components/molecules/indentity'
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
    <Card className="overflow-hidden border-border/60 bg-gradient-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-6 md:px-6">
        <div className="space-y-4">
          <div className="flex flex-wrap items-start gap-4">
            <div className="rounded-2xl border border-border/60 bg-background/75 p-4 shadow-sm backdrop-blur">
              <Identity
                className="text-lg"
                color={employee.color}
                icon={employee.icon}
                name={employee.name || 'Untitled employee'}
                size="lg"
              />
            </div>
            <div className="min-w-0 flex-1 space-y-2">
              <div className="space-y-1">
                <h1 className="truncate text-3xl font-medium tracking-tight">{employee.name || 'Untitled employee'}</h1>
                <p className="text-base text-muted-foreground">
                  {employee.title ||
                    (viewmodel.isOwnerEmployee
                      ? 'Update your display identity for the workspace.'
                      : 'Define this employee’s role, behavior, and operating context.')}
                </p>
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
          </div>
          <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
            This page saves in place. Make changes directly to profile, provider, and runtime settings without switching
            into a separate edit mode.
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
