import { useNavigate } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useAppViewmodel } from '@/app.viewmodel'
import { Page } from '@/components/layouts/page'
import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { RunStatusFilter } from '@/lib/api/runs'
import { AppModel } from '@/models/app.model'
import type { RunsPageViewmodel } from './runs.viewmodel'

interface RunsPageProps {
  viewmodel: RunsPageViewmodel
}

const ALL_EMPLOYEES = '__all_employees__'
const ALL_STATUSES = '__all_statuses__'

const STATUS_OPTIONS: Array<{ label: string, value: RunStatusFilter }> = [
  { label: 'Pending', value: 'Pending' },
  { label: 'Running', value: 'Running' },
  { label: 'Completed', value: 'Completed' },
  { label: 'Cancelled', value: 'Cancelled' },
  { label: 'Failed', value: 'Failed' },
]

const buildRunsLocation = ({
  employeeId,
  page,
  status,
}: {
  employeeId: string | null
  page: number
  status: RunStatusFilter | null
}) => {
  const params = new URLSearchParams()

  if (page > 1) {
    params.set('page', page.toString())
  }

  if (employeeId && employeeId !== 'null') {
    params.set('employee', employeeId)
  }

  if (status) {
    params.set('status', status)
  }

  const search = params.toString()
  return search ? `/runs?${search}` : '/runs'
}

export const RunsPage = observer(({ viewmodel }: RunsPageProps) => {
  const navigate = useNavigate()
  const appViewmodel = useAppViewmodel()
  const employees = AppModel.instance.employees
  const selectedEmployeeName = viewmodel.filters.employeeId
    ? employees.find((employee) => employee.id === viewmodel.filters.employeeId)?.name ?? 'Unknown employee'
    : 'All employees'
  const selectedStatusLabel = STATUS_OPTIONS.find((option) => option.value === viewmodel.filters.status)?.label ?? 'All statuses'

  const updateSearch = (next: { employeeId?: string | null, page?: number, status?: RunStatusFilter | null }) => {
    const employeeId = next.employeeId === undefined ? viewmodel.filters.employeeId : next.employeeId
    const status = next.status === undefined ? viewmodel.filters.status : next.status

    void navigate({
      to: buildRunsLocation({
        employeeId,
        page: next.page ?? viewmodel.page,
        status,
      }),
    })
  }

  const emptyStateMessage = viewmodel.hasActiveFilters
    ? 'No runs match the current filters. Try a different employee or status, or clear the filters to see all runs.'
    : 'No runs found for this page.'

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Runs</CardTitle>
            <CardDescription>
              Browse historical runs across all employees and open one to inspect its full timeline.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex flex-col gap-2">
                  <span className="text-sm font-medium">Employee</span>
                  <Select
                    value={viewmodel.filters.employeeId ?? ALL_EMPLOYEES}
                    onValueChange={(value) => updateSearch({ employeeId: value === ALL_EMPLOYEES ? null : value, page: 1 })}
                  >
                    <SelectTrigger className="w-full min-w-52">
                      <SelectValue>{selectedEmployeeName}</SelectValue>
                    </SelectTrigger>
                    <SelectContent align="start">
                      <SelectItem value={ALL_EMPLOYEES}>All employees</SelectItem>
                      {employees.map((employee) => (
                        <SelectItem key={employee.id} value={employee.id}>
                          {employee.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex flex-col gap-2">
                  <span className="text-sm font-medium">Status</span>
                  <Select
                    value={viewmodel.filters.status ?? ALL_STATUSES}
                    onValueChange={(value) =>
                      updateSearch({ page: 1, status: value === ALL_STATUSES ? null : (value as RunStatusFilter) })
                    }
                  >
                    <SelectTrigger className="w-full min-w-44">
                      <SelectValue>{selectedStatusLabel}</SelectValue>
                    </SelectTrigger>
                    <SelectContent align="start">
                      <SelectItem value={ALL_STATUSES}>All statuses</SelectItem>
                      {STATUS_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="flex items-center gap-2">
                <Button disabled={!viewmodel.hasActiveFilters} variant="ghost" onClick={() => updateSearch({ employeeId: null, page: 1, status: null })}>
                  Clear filters
                </Button>
              </div>
            </div>

            <div className="flex flex-col gap-3 text-sm text-muted-foreground md:flex-row md:items-center md:justify-between">
              <span>{viewmodel.total} total runs</span>
              <div className="flex items-center gap-2">
                <Button disabled={viewmodel.page <= 1} variant="outline" onClick={() => updateSearch({ page: viewmodel.page - 1 })}>
                  Previous
                </Button>
                <span>
                  Page {viewmodel.page} of {viewmodel.totalPages}
                </span>
                <Button disabled={viewmodel.page >= viewmodel.totalPages} variant="outline" onClick={() => updateSearch({ page: viewmodel.page + 1 })}>
                  Next
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {viewmodel.errorMessage && (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        )}

        {viewmodel.items.map((run) => (
          <RunSummaryCard key={run.id} latestActivity={appViewmodel.runs.latestActivity(run.id)} run={run} />
        ))}

        {viewmodel.items.length === 0 && (
          <Card>
            <CardContent className="flex flex-col gap-3 py-6 text-sm text-muted-foreground">
              <span>{emptyStateMessage}</span>
              {viewmodel.hasActiveFilters ? (
                <div>
                  <Button variant="outline" onClick={() => updateSearch({ employeeId: null, page: 1, status: null })}>
                    Reset filters
                  </Button>
                </div>
              ) : null}
            </CardContent>
          </Card>
        )}
      </div>
    </Page>
  )
})
