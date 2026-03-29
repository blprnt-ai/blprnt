import { useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { useAppViewmodel } from '@/app.viewmodel'
import { RunPage } from './run.page'
import { RunPageViewmodel } from './run.viewmodel'

export const RunProvider = () => {
  const { runId } = useParams({ from: '/runs/$runId/' })
  const appViewmodel = useAppViewmodel()
  const [viewmodel, setViewmodel] = useState(() => new RunPageViewmodel(runId, appViewmodel.runs))

  useEffect(() => {
    const nextViewmodel = new RunPageViewmodel(runId, appViewmodel.runs)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [appViewmodel.runs, runId])

  if (viewmodel.isLoading) return <AppLoader />

  return <RunPage viewmodel={viewmodel} />
}
