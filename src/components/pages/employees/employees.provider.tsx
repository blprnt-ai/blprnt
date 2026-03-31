import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { EmployeesPage } from './employees.page'
import { EmployeesViewmodel, EmployeesViewmodelContext } from './employees.viewmodel'

export const EmployeesProvider = observer(() => {
  const [viewmodel] = useState(() => new EmployeesViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <EmployeesViewmodelContext.Provider value={viewmodel}>
      <EmployeesPage />
    </EmployeesViewmodelContext.Provider>
  )
})
