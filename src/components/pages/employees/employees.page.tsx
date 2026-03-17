import { Page } from '@/components/layouts/page'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { EmployeesDirectory } from './components/employees-directory'
import { useEmployeesViewmodel } from './employees.viewmodel'

export const EmployeesPage = () => {
  const viewmodel = useEmployeesViewmodel()

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <Card>
          <CardHeader>
            <CardTitle>Employees</CardTitle>
            <CardDescription>
              Browse the people and agents in your workspace, then open one to review or edit its configuration.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {viewmodel.employees.length} {viewmodel.employees.length === 1 ? 'employee' : 'employees'}
            </p>
          </CardContent>
        </Card>

        <EmployeesDirectory />
      </div>
    </Page>
  )
}
