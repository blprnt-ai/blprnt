import { Link } from '@tanstack/react-router'
import { BriefcaseIcon, CrownIcon, UserRoundCogIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { OrgChart } from '@/bindings/OrgChart'
import { EmptyState } from '@/components/pages/issue/components/empty-state'
import { Card, CardContent } from '@/components/ui/card'
import { formatCapabilities, formatRole } from '../../employee/utils'
import { useEmployeesViewmodel } from '../employees.viewmodel'

export const EmployeesOrgChart = observer(() => {
  const viewmodel = useEmployeesViewmodel()

  if (viewmodel.orgChart.length === 0) {
    return (
      <EmptyState description="Employees with reporting relationships will appear here." title="No org chart yet" />
    )
  }

  return (
    <Card className="overflow-hidden border-border/60 py-0">
      <CardContent className="overflow-x-auto px-5 py-6 md:px-6">
        <div className="flex min-w-max justify-center px-6">
          <div className="flex flex-wrap justify-center gap-10">
            {viewmodel.orgChart.map((node) => (
              <OrgChartNode key={node.id} node={node} />
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  )
})

const OrgChartNode = ({ node }: { node: OrgChart }) => {
  return (
    <div className="flex flex-col items-center">
      <Link params={{ employeeId: node.id }} to="/employees/$employeeId" className="w-72">
        <Card className="border-border/60 bg-background/85 py-0 transition-colors hover:bg-muted/40">
          <CardContent className="space-y-4 px-5 py-4">
            <div className="flex items-start gap-3">
              <div className="flex size-11 shrink-0 items-center justify-center rounded-full border border-border/60 bg-muted/50 text-muted-foreground">
                <RoleIcon role={node.role} />
              </div>
              <div className="min-w-0 flex-1 space-y-1">
                <p className="truncate font-medium">{node.name}</p>
                <p className="text-sm text-muted-foreground">{node.title || formatRole(node.role)}</p>
              </div>
              <span className="rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground">{node.status}</span>
            </div>

            <div className="space-y-1 text-sm text-muted-foreground">
              <p>{formatRole(node.role)}</p>
              <p className="line-clamp-2">{formatCapabilities(node.capabilities)}</p>
            </div>
          </CardContent>
        </Card>
      </Link>

      {node.reports.length > 0 ? (
        <div className="mt-4 flex flex-col items-center">
          <div className="h-6 w-px bg-border/70" />
          <div className="h-px w-full min-w-24 bg-border/70" />
          <div className="flex items-start justify-center gap-8 pt-6">
            {node.reports.map((report) => (
              <div key={report.id} className="relative flex flex-col items-center">
                <div className="absolute -top-6 h-6 w-px bg-border/70" />
                <OrgChartNode node={report} />
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  )
}

const RoleIcon = ({ role }: { role: OrgChart['role'] }) => {
  if (role === 'owner') return <CrownIcon className="size-5" />
  if (role === 'manager') return <BriefcaseIcon className="size-5" />
  return <UserRoundCogIcon className="size-5" />
}
