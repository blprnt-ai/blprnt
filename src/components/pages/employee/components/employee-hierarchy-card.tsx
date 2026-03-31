import type { ComponentType } from 'react'
import { Building2, UserRoundCheck } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { AppModel } from '@/models/app.model'
import { cn } from '@/lib/utils'
import { useEmployeeViewmodel } from '../employee.viewmodel'

interface EmployeeHierarchyCardProps {
  compact?: boolean
}

export const EmployeeHierarchyCard = observer(({ compact = false }: EmployeeHierarchyCardProps) => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const reportsTo = AppModel.instance.resolveEmployeeName(viewmodel.reportsTo) ?? 'No manager assigned'
  const chainOfCommand =
    viewmodel.chainOfCommand.length > 0 ? viewmodel.chainOfCommand.map((entry) => entry.name) : ['No chain of command']

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Hierarchy</CardTitle>
        <CardDescription>Reference information for reporting lines and organizational placement.</CardDescription>
      </CardHeader>
      <CardContent className={cn('space-y-4', compact && 'pb-6')}>
        <HierarchyRow icon={UserRoundCheck} label="Reports to" value={reportsTo} />

        <div className="space-y-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
          <div className="flex items-center gap-3">
            <div className="flex size-9 items-center justify-center rounded-full bg-background text-muted-foreground">
              <Building2 className="size-4" />
            </div>
            <div>
              <p className="text-sm font-medium">Chain of command</p>
              <p className="text-sm text-muted-foreground">
                Visible context only. This page does not edit reporting lines.
              </p>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {chainOfCommand.map((entry) => (
              <span key={entry} className="rounded-full border border-border/60 bg-background px-3 py-1.5 text-sm">
                {entry}
              </span>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  )
})

const HierarchyRow = ({
  icon: Icon,
  label,
  value,
}: {
  icon: ComponentType<{ className?: string }>
  label: string
  value: string
}) => {
  return (
    <div className="rounded-2xl border border-border/60 bg-muted/20 p-4">
      <div className="flex items-center gap-3">
        <div className="flex size-9 items-center justify-center rounded-full bg-background text-muted-foreground">
          <Icon className="size-4" />
        </div>
        <div className="min-w-0">
          <p className="text-sm font-medium">{label}</p>
          <p className="break-words text-sm text-muted-foreground">{value}</p>
        </div>
      </div>
    </div>
  )
}
