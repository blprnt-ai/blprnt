import { useNavigate, useParams } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { useAppViewmodel } from '@/app.viewmodel'
import { RunPage } from './run.page'
import { RunPageViewmodel } from './run.viewmodel'

export const RunProvider = observer(() => {
  const { runId } = useParams({ from: '/runs/$runId/' })
  const navigate = useNavigate()
  const appViewmodel = useAppViewmodel()
  const [viewmodel, setViewmodel] = useState(
    () =>
      new RunPageViewmodel({
        onRunCreated: async (nextRunId) => {
          await navigate({
            params: { runId: nextRunId },
            to: '/runs/$runId',
          })
        },
        runId,
        runs: appViewmodel.runs,
      }),
  )

  useEffect(() => {
    const nextViewmodel = new RunPageViewmodel({
      onRunCreated: async (nextRunId) => {
        await navigate({
          params: { runId: nextRunId },
          to: '/runs/$runId',
        })
      },
      runId,
      runs: appViewmodel.runs,
    })
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [appViewmodel.runs, navigate, runId])

  if (viewmodel.isLoading) return <AppLoader />

  return <RunPage viewmodel={viewmodel} />
})
