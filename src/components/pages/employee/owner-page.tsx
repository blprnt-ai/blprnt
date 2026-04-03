import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { EmployeeHeader } from './components/employee-header'
import { EmployeeNotFound } from './components/employee-not-found'
import { EmployeeProfileTab } from './components/employee-profile-tab'
import { useEmployeeViewmodel } from './employee.viewmodel'

export const OwnerPage = observer(() => {
  const viewmodel = useEmployeeViewmodel()

  if (!viewmodel.employee) return <EmployeeNotFound />

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5 h-full">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <EmployeeHeader />

        <EmployeeProfileTab />
      </div>
    </Page>
  )
})
