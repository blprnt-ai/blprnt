import { useRouterState } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { AppLoader } from '@/components/organisms/app-loader'
import type { RunStatusFilter } from '@/lib/api/runs'
import { RunsPage } from './runs.page'
import { RunsPageViewmodel } from './runs.viewmodel'

const RUN_STATUS_FILTERS: RunStatusFilter[] = ['Pending', 'Running', 'Completed', 'Cancelled', 'Failed']

const normalizeQueryValue = (value: string | null) => {
  if (!value || value === 'null') return null

  return value
}

const readQuery = (search: string) => {
  const params = new URLSearchParams(search)
  const value = Number.parseInt(params.get('page') ?? '1', 10)
  const status = normalizeQueryValue(params.get('status'))

  return {
    employeeId: normalizeQueryValue(params.get('employee')),
    page: Number.isNaN(value) ? 1 : Math.max(1, value),
    status: status && RUN_STATUS_FILTERS.includes(status as RunStatusFilter) ? (status as RunStatusFilter) : null,
  }
}

export const RunsProvider = observer(() => {
  const appViewmodel = useAppViewmodel()
  const search = useRouterState({ select: (state) => state.location.searchStr })
  const query = readQuery(search)
  const [viewmodel, setViewmodel] = useState(() => new RunsPageViewmodel(appViewmodel.runs, query.page, query))

  useEffect(() => {
    const nextViewmodel = new RunsPageViewmodel(appViewmodel.runs, query.page, query)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [appViewmodel.runs, query.employeeId, query.page, query.status])

  if (viewmodel.isLoading) return <AppLoader />

  return <RunsPage viewmodel={viewmodel} />
})
