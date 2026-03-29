import { Page } from '@/components/layouts/page'
import { EmployeeHeader } from './components/employee-header'
import { EmployeeIdentityCard } from './components/employee-identity-card'
import { EmployeeNotFound } from './components/employee-not-found'
import { EmployeeRuntimeCard } from './components/employee-runtime-card'
import { useEmployeeViewmodel } from './employee.viewmodel'

export const EmployeePage = () => {
  const viewmodel = useEmployeeViewmodel()
  const isAgent = viewmodel.showsAgentConfiguration

  if (!viewmodel.employee) return <EmployeeNotFound />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <EmployeeHeader />
        {isAgent ? (
          <div className="grid gap-4">
            <EmployeeRuntimeCard />

            <EmployeeIdentityCard />
          </div>
        ) : (
          <EmployeeIdentityCard />
        )}
      </div>
    </Page>
  )
}
