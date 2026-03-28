import { Page } from '@/components/layouts/page'
import { EmployeeAppearanceCard } from './components/employee-appearance-card'
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

  if (!viewmodel.employee) return <EmployeeNotFound />

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <EmployeeHeader />
        <div className="flex flex-col gap-3 lg:flex-row lg:justify-between">
          <div className="flex min-w-0 flex-col gap-3">
            <EmployeeIdentityCard />
            <EmployeeAppearanceCard />
            <EmployeeCapabilitiesCard />
          </div>
          <div className="flex w-full flex-col gap-3 lg:w-[320px]">
            <EmployeeHierarchyCard />
            {viewmodel.showsAgentConfiguration ? (
              <>
                <EmployeeConnectionCard />
                <EmployeeRuntimeCard />
              </>
            ) : null}
          </div>
        </div>
      </div>
    </Page>
  )
}
