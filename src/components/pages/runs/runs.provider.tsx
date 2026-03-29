import { useEffect, useState } from 'react'
import { useRouterState } from '@tanstack/react-router'
import { AppLoader } from '@/components/organisms/app-loader'
import { useAppViewmodel } from '@/app.viewmodel'
import { RunsPage } from './runs.page'
import { RunsPageViewmodel } from './runs.viewmodel'

const readPage = (search: string) => {
  const value = Number.parseInt(new URLSearchParams(search).get('page') ?? '1', 10)
  return Number.isNaN(value) ? 1 : Math.max(1, value)
}

export const RunsProvider = () => {
  const appViewmodel = useAppViewmodel()
  const search = useRouterState({ select: (state) => state.location.searchStr })
  const page = readPage(search)
  const [viewmodel, setViewmodel] = useState(() => new RunsPageViewmodel(appViewmodel.runs, page))

  useEffect(() => {
    const nextViewmodel = new RunsPageViewmodel(appViewmodel.runs, page)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [appViewmodel.runs, page])

  if (viewmodel.isLoading) return <AppLoader />

  return <RunsPage viewmodel={viewmodel} />
}
