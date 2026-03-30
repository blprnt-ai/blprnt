import { useNavigate, useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { EmployeePage } from './employee.page'
import { EmployeeViewmodel, EmployeeViewmodelContext } from './employee.viewmodel'

export const EmployeeProvider = () => {
  const { employeeId } = useParams({ from: '/employees/$employeeId/' })
  const navigate = useNavigate()
  const [viewmodel, setViewmodel] = useState(
    () =>
      new EmployeeViewmodel(employeeId, {
        onIssueCreated: async (issue) => {
          await navigate({
            params: { issueId: issue.id },
            to: '/issues/$issueId',
          })
        },
        onTerminated: async () => {
          await navigate({ to: '/employees' })
        },
      }),
  )

  useEffect(() => {
    const nextViewmodel = new EmployeeViewmodel(employeeId, {
      onIssueCreated: async (issue) => {
        await navigate({
          params: { issueId: issue.id },
          to: '/issues/$issueId',
        })
      },
      onTerminated: async () => {
        await navigate({ to: '/employees' })
      },
    })
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()

    return () => {
      nextViewmodel.destroy()
    }
  }, [employeeId, navigate])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <EmployeeViewmodelContext.Provider value={viewmodel}>
      <EmployeePage />
    </EmployeeViewmodelContext.Provider>
  )
}
