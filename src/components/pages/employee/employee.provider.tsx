import { useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { EmployeePage } from './employee.page'
import { EmployeeViewmodel, EmployeeViewmodelContext } from './employee.viewmodel'

export const EmployeeProvider = () => {
  const { employeeId } = useParams({ from: '/employees/$employeeId/' })
  const [viewmodel, setViewmodel] = useState(() => new EmployeeViewmodel(employeeId))

  useEffect(() => {
    const nextViewmodel = new EmployeeViewmodel(employeeId)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()

    return () => {
      nextViewmodel.destroy()
    }
  }, [employeeId])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <EmployeeViewmodelContext.Provider value={viewmodel}>
      <EmployeePage />
    </EmployeeViewmodelContext.Provider>
  )
}
