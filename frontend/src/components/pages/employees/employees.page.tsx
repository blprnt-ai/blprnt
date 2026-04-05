import { useNavigate } from '@tanstack/react-router'
import { NetworkIcon, Rows3Icon, UserPlusIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { EmployeeForm } from '@/components/forms/employee'
import { EmployeeFormViewmodel } from '@/components/forms/employee/employee-form.viewmodel'
import { Page } from '@/components/layouts/page'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { EmployeeImportCard } from './components/employee-import-card'
import { EmployeesDirectory } from './components/employees-directory'
import { EmployeesOrgChart } from './components/employees-org-chart'
import { useEmployeesViewmodel } from './employees.viewmodel'

export const EmployeesPage = observer(() => {
  const viewmodel = useEmployeesViewmodel()
  const navigate = useNavigate()
  const [employeeFormViewmodel] = useState(
    () =>
      new EmployeeFormViewmodel(async (employee) => {
        await navigate({
          params: { employeeId: employee.id },
          to: '/employees/$employeeId',
        })
      }),
  )

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <div className="flex justify-end">
          <Button type="button" variant="secondary" onClick={employeeFormViewmodel.open}>
            <UserPlusIcon />
            Add employee
          </Button>
        </div>

        <EmployeeImportCard />

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <Tabs
          value={viewmodel.activeView}
          onValueChange={(value) => viewmodel.setActiveView(value as 'list' | 'org-chart')}
        >
          <TabsList variant="line">
            <TabsTrigger value="list">
              <Rows3Icon className="size-4" />
              List
            </TabsTrigger>
            <TabsTrigger value="org-chart">
              <NetworkIcon className="size-4" />
              Org chart
            </TabsTrigger>
          </TabsList>

          <TabsContent className="mt-5" value="list">
            <EmployeesDirectory />
          </TabsContent>

          <TabsContent className="mt-5" value="org-chart">
            <EmployeesOrgChart />
          </TabsContent>
        </Tabs>

        <EmployeeForm viewmodel={employeeFormViewmodel} />
      </div>
    </Page>
  )
})
