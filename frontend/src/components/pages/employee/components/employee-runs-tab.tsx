import { observer } from 'mobx-react-lite'
import { useAppViewmodel } from '@/app.viewmodel'
import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeRunsTab = observer(() => {
  const viewmodel = useEmployeeViewmodel()
  const appViewmodel = useAppViewmodel()

  if (viewmodel.isRunsLoading) {
    return (
      <Card>
        <CardContent className="py-6 text-sm text-muted-foreground">Loading runs...</CardContent>
      </Card>
    )
  }

  return (
    <div className="grid gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Runs</CardTitle>
          <CardDescription>{viewmodel.runSummaries.length} recent runs for this employee.</CardDescription>
        </CardHeader>
      </Card>

      {viewmodel.runsErrorMessage ? (
        <Card>
          <CardContent className="py-4 text-sm text-destructive">{viewmodel.runsErrorMessage}</CardContent>
        </Card>
      ) : null}

      {viewmodel.runSummaries.map((run) => (
        <RunSummaryCard key={run.id} latestActivity={appViewmodel.runs.latestActivity(run.id)} run={run} />
      ))}

      {!viewmodel.runsErrorMessage && viewmodel.runSummaries.length === 0 ? (
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">No runs yet for this employee.</CardContent>
        </Card>
      ) : null}
    </div>
  )
})
