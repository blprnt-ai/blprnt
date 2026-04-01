import { Link } from '@tanstack/react-router'
import { ArrowUpRightIcon } from 'lucide-react'
import type { Employee } from '@/bindings/Employee'
import { IdentityLink } from '@/components/molecules/indentity'
import { formatCapabilities, formatRole } from '@/components/pages/employee/utils'
import { Card, CardContent } from '@/components/ui/card'
import type { ColorVariant } from '@/components/ui/colors'

interface EmployeeListItemProps {
  employee: Employee
}

export const EmployeeListItem = ({ employee }: EmployeeListItemProps) => {
  return (
    <Link
      className="group block w-full md:w-[calc(50%-0.5rem)] xl:w-[calc(33.333%-0.75rem)]"
      params={{ employeeId: employee.id }}
      to="/employees/$employeeId"
    >
      <Card className="border-border/60 py-0 transition-all hover:-translate-y-0.5 hover:bg-muted/30">
        <CardContent className="flex flex-col gap-5 px-5 py-5">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-2">
              <IdentityLink
                color={employee.color as ColorVariant}
                employeeId={employee.id}
                icon={employee.icon}
                name={employee.name}
                size="lg"
              />
              <p className="text-sm text-muted-foreground">{employee.title || formatRole(employee.role)}</p>
            </div>
            <div className="flex items-center gap-2">
              <span className="rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground">{employee.status}</span>
              <ArrowUpRightIcon className="size-4 text-muted-foreground transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
            </div>
          </div>

          <div className="grid gap-3 text-sm">
            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Role</p>
              <p className="mt-2">{formatRole(employee.role)}</p>
            </div>
            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Capabilities</p>
              <p className="mt-2 line-clamp-3 text-muted-foreground">{formatCapabilities(employee.capabilities)}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}
