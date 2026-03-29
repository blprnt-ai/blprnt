import { useNavigate } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import type { RunsPageViewmodel } from './runs.viewmodel'
import { useAppViewmodel } from '@/app.viewmodel'

interface RunsPageProps {
  viewmodel: RunsPageViewmodel
}

export const RunsPage = observer(({ viewmodel }: RunsPageProps) => {
  const navigate = useNavigate()
  const appViewmodel = useAppViewmodel()

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <Card>
          <CardHeader>
            <CardTitle>Runs</CardTitle>
            <CardDescription>Browse historical runs across all employees and open one to inspect its full timeline.</CardDescription>
          </CardHeader>
          <CardContent className="flex items-center justify-between gap-4 text-sm text-muted-foreground">
            <span>{viewmodel.total} total runs</span>
            <div className="flex items-center gap-2">
              <Button
                disabled={viewmodel.page <= 1}
                variant="outline"
                onClick={() => void navigate({ to: '/runs', search: { page: viewmodel.page - 1 } as never })}
              >
                Previous
              </Button>
              <span>
                Page {viewmodel.page} of {viewmodel.totalPages}
              </span>
              <Button
                disabled={viewmodel.page >= viewmodel.totalPages}
                variant="outline"
                onClick={() => void navigate({ to: '/runs', search: { page: viewmodel.page + 1 } as never })}
              >
                Next
              </Button>
            </div>
          </CardContent>
        </Card>

        {viewmodel.errorMessage && <Card><CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent></Card>}

        {viewmodel.items.map((run) => (
          <RunSummaryCard key={run.id} latestActivity={appViewmodel.runs.latestActivity(run.id)} run={run} />
        ))}

        {viewmodel.items.length === 0 && (
          <Card>
            <CardContent className="py-6 text-sm text-muted-foreground">No runs found for this page.</CardContent>
          </Card>
        )}
      </div>
    </Page>
  )
})
