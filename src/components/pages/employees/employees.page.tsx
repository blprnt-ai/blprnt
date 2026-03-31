import { NetworkIcon, Rows3Icon } from 'lucide-react'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { EmployeeImportCard } from './components/employee-import-card'
import { EmployeesDirectory } from './components/employees-directory'
import { EmployeesOrgChart } from './components/employees-org-chart'
import { useEmployeesViewmodel } from './employees.viewmodel'

export const EmployeesPage = () => {
  const viewmodel = useEmployeesViewmodel()

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <EmployeeImportCard />

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <Tabs value={viewmodel.activeView} onValueChange={(value) => viewmodel.setActiveView(value as 'list' | 'org-chart')}>
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
      </div>
    </Page>
  )
}
