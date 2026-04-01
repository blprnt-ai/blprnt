import { useNavigate, useParams } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { AppLoader } from '@/components/organisms/app-loader'
import { EmployeePage } from './employee.page'
import { EmployeeViewmodel, EmployeeViewmodelContext } from './employee.viewmodel'

export const EmployeeProvider = observer(() => {
  const { employeeId } = useParams({ from: '/employees/$employeeId/' })
  const navigate = useNavigate()
  const appViewmodel = useAppViewmodel()
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
        onRunCreated: async (runId) => {
          await navigate({
            params: { runId },
            to: '/runs/$runId',
          })
        },
        onTerminated: async () => {
          await navigate({ to: '/employees' })
        },
        runs: appViewmodel.runs,
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
      onRunCreated: async (runId) => {
        await navigate({
          params: { runId },
          to: '/runs/$runId',
        })
      },
      onTerminated: async () => {
        await navigate({ to: '/employees' })
      },
      runs: appViewmodel.runs,
    })
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()

    return () => {
      nextViewmodel.destroy()
    }
  }, [appViewmodel.runs, employeeId, navigate])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <EmployeeViewmodelContext.Provider value={viewmodel}>
      <EmployeePage />
    </EmployeeViewmodelContext.Provider>
  )
})
