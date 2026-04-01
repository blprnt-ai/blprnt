import { observer } from 'mobx-react-lite'
import { IssueForm } from '@/components/forms/issue'
import { Page } from '@/components/layouts/page'
import { EmployeeHeader } from './components/employee-header'
import { EmployeeHierarchyCard } from './components/employee-hierarchy-card'
import { EmployeeIdentityCard } from './components/employee-identity-card'
import { EmployeeNotFound } from './components/employee-not-found'
import { EmployeeRuntimeCard } from './components/employee-runtime-card'
import { EmployeeSkillStackCard } from './components/employee-skill-stack-card'
import { useEmployeeViewmodel } from './employee.viewmodel'

export const EmployeePage = observer(() => {
  const viewmodel = useEmployeeViewmodel()
  const isAgent = viewmodel.showsAgentConfiguration

  if (!viewmodel.employee) return <EmployeeNotFound />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5 h-full">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <EmployeeHeader />
        {isAgent ? (
          <div className="grid gap-4">
            <EmployeeRuntimeCard />
            <EmployeeSkillStackCard />
            <EmployeeIdentityCard />
            <EmployeeHierarchyCard />
          </div>
        ) : (
          <div className="grid gap-4">
            <EmployeeIdentityCard />
            <EmployeeHierarchyCard />
          </div>
        )}
      </div>
      <IssueForm viewmodel={viewmodel.issueFormViewmodel} />
    </Page>
  )
})
