import { Link } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useAppViewmodel } from '@/app.viewmodel'

export const DashboardPage = observer(() => {
  const appViewmodel = useAppViewmodel()
  const recentRuns = appViewmodel.runs.recentRuns

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <Card>
          <CardHeader>
            <CardTitle>Recent Runs</CardTitle>
            <CardDescription>
              Track the 5 latest runs across all employees. Running work updates live without polling.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {recentRuns.length === 0 && (
              <p className="text-sm text-muted-foreground">No runs yet. Trigger one from the runs page or assign an issue.</p>
            )}
            {recentRuns.map((run) => (
              <RunSummaryCard key={run.id} latestActivity={appViewmodel.runs.latestActivity(run.id)} run={run} />
            ))}
            <div className="flex justify-end">
              <Link to="/runs">
                <Button variant="outline">View all runs</Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    </Page>
  )
})
