import { Link } from '@tanstack/react-router'
import type { Employee } from '@/bindings/Employee'
import { Identity } from '@/components/molecules/indentity'
import { formatCapabilities, formatRole } from '@/components/pages/employee/utils'
import { Card, CardContent } from '@/components/ui/card'
import type { ColorVariant } from '@/components/ui/colors'

interface EmployeeListItemProps {
  employee: Employee
}

export const EmployeeListItem = ({ employee }: EmployeeListItemProps) => {
  return (
    <Link params={{ employeeId: employee.id }} to="/employees/$employeeId">
      <Card className="transition-colors hover:bg-muted/40">
        <CardContent className="flex flex-col gap-4">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-1">
              <Identity color={employee.color as ColorVariant} icon={employee.icon} name={employee.name} size="lg" />
              <p className="text-sm text-muted-foreground">{employee.title || 'No title'}</p>
            </div>
            <span className="rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground">{employee.status}</span>
          </div>
          <div className="space-y-1 text-sm text-muted-foreground">
            <p>{formatRole(employee.role)}</p>
            <p>{formatCapabilities(employee.capabilities)}</p>
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}
