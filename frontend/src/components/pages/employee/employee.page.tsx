import { Cpu, ScrollText, UserRound } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { IssueForm } from '@/components/forms/issue'
import { Page } from '@/components/layouts/page'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { EmployeeHeader } from './components/employee-header'
import { EmployeeLifeTab } from './components/employee-life-tab'
import { EmployeeNotFound } from './components/employee-not-found'
import { EmployeeProfileTab } from './components/employee-profile-tab'
import { EmployeeRuntimeTab } from './components/employee-runtime-tab'
import { useEmployeeViewmodel } from './employee.viewmodel'
import { OwnerPage } from './owner-page'

export const EmployeePage = observer(() => {
  const viewmodel = useEmployeeViewmodel()

  if (!viewmodel.employee) return <EmployeeNotFound />

  if (viewmodel.employee.role === 'owner') return <OwnerPage />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5 h-full">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <EmployeeHeader />

        <Tabs
          value={viewmodel.activeTab}
          onValueChange={(value) => viewmodel.setActiveTab(value as typeof viewmodel.activeTab)}
        >
          <TabsList variant="line">
            <TabsTrigger value="profile">
              <UserRound className="size-4" />
              Profile
            </TabsTrigger>
            <TabsTrigger value="runtime">
              <Cpu className="size-4" />
              Runtime
            </TabsTrigger>
            <TabsTrigger value="life">
              <ScrollText className="size-4" />
              Life
            </TabsTrigger>
          </TabsList>

          <TabsContent className="mt-5" value="profile">
            <EmployeeProfileTab />
          </TabsContent>

          <TabsContent className="mt-5" value="runtime">
            <EmployeeRuntimeTab />
          </TabsContent>

          <TabsContent className="mt-5" value="life">
            <EmployeeLifeTab />
          </TabsContent>
        </Tabs>
      </div>
      <IssueForm viewmodel={viewmodel.issueFormViewmodel} />
    </Page>
  )
})
