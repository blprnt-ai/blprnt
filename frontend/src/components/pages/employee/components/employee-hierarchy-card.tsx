import { Building2, ChevronRight } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { IdentityLink } from '@/components/molecules/indentity'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { ColorVariant } from '@/components/ui/colors'
import { cn } from '@/lib/utils'
import { useEmployeeViewmodel } from '../employee.viewmodel'

interface EmployeeHierarchyCardProps {
  compact?: boolean
}

export const EmployeeHierarchyCard = observer(({ compact = false }: EmployeeHierarchyCardProps) => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  const chainOfCommand =
    viewmodel.chainOfCommand.length > 0
      ? viewmodel.chainOfCommand.map((entry) => ({
          color: entry.color,
          icon: entry.icon,
          id: entry.id,
          name: entry.name,
        }))
      : null

  if (!chainOfCommand) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>
          <div className="flex items-center gap-3">
            <div className="flex size-9 items-center justify-center rounded-full bg-background text-muted-foreground">
              <Building2 className="size-4" />
            </div>
            <div>
              <p className="text-sm font-medium">Chain of command</p>
            </div>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className={cn('flex items-center gap-2', compact && 'pb-6')}>
        {chainOfCommand.map((entry, idx) => (
          <div key={entry.id} className="flex items-center gap-2">
            <IdentityLink
              color={entry.color as ColorVariant}
              employeeId={entry.id}
              icon={entry.icon}
              name={entry.name}
            />

            {idx < chainOfCommand.length - 1 && <ChevronRight className="size-4" />}
          </div>
        ))}
      </CardContent>
    </Card>
  )
})
