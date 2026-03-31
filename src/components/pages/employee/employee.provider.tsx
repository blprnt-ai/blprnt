import { useNavigate, useParams } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { EmployeePage } from './employee.page'
import { EmployeeViewmodel, EmployeeViewmodelContext } from './employee.viewmodel'

export const EmployeeProvider = observer(() => {
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
        onOpenChat: async (nextEmployeeId) => {
          await navigate({
            params: { employeeId: nextEmployeeId },
            to: '/employees/$employeeId/chat',
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
      onOpenChat: async (nextEmployeeId) => {
        await navigate({
          params: { employeeId: nextEmployeeId },
          to: '/employees/$employeeId/chat',
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
})
