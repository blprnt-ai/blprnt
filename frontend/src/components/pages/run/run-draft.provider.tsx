import { useNavigate, useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { RunPage } from './run.page'
import { RunPageViewmodel } from './run.viewmodel'

export const RunDraftProvider = () => {
  const { employeeId } = useParams({ from: '/employees/$employeeId/chat' })
  const navigate = useNavigate()
  const appViewmodel = useAppViewmodel()
  const [viewmodel, setViewmodel] = useState(
    () =>
      new RunPageViewmodel({
        employeeId,
        onRunCreated: async (runId) => {
          await navigate({
            params: { runId },
            to: '/runs/$runId',
          })
        },
        runs: appViewmodel.runs,
      }),
  )

  useEffect(() => {
    const nextViewmodel = new RunPageViewmodel({
      employeeId,
      onRunCreated: async (runId) => {
        await navigate({
          params: { runId },
          to: '/runs/$runId',
        })
      },
      runs: appViewmodel.runs,
    })
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [appViewmodel.runs, employeeId, navigate])

  return <RunPage viewmodel={viewmodel} />
}
