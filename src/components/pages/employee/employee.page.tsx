import { Page } from '@/components/layouts/page'
import { EmployeeCapabilitiesCard } from './components/employee-capabilities-card'
import { EmployeeConnectionCard } from './components/employee-connection-card'
import { EmployeeHeader } from './components/employee-header'
import { EmployeeHierarchyCard } from './components/employee-hierarchy-card'
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
            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_360px]">
              <div className="grid min-w-0 gap-4">
                <EmployeeIdentityCard />
                <div className="grid gap-4 lg:grid-cols-2">
                  <EmployeeCapabilitiesCard />
                  <EmployeeConnectionCard />
                </div>
              </div>
              <EmployeeHierarchyCard />
            </div>
          </div>
        ) : (
          <div className="grid gap-4 lg:grid-cols-2">
            <EmployeeIdentityCard />
            <EmployeeCapabilitiesCard />
            <EmployeeHierarchyCard compact />
          </div>
        )}
      </div>
    </Page>
  )
}
