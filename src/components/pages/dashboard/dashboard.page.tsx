import { ActivityIcon, BotIcon, CircleDashedIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useAppViewmodel } from '@/app.viewmodel'
import { Page } from '@/components/layouts/page'
import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DashboardActivityChart } from './components/dashboard-activity-chart'
import { DashboardMetricCard } from './components/dashboard-metric-card'
import { DashboardProjectHealth } from './components/dashboard-project-health'
import { DashboardWorkloadCard } from './components/dashboard-workload-card'
import { useDashboardViewmodel } from './dashboard.viewmodel'

export const DashboardPage = observer(() => {
  const viewmodel = useDashboardViewmodel()
  const appViewmodel = useAppViewmodel()

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <DashboardMetricCard icon={CircleDashedIcon} label="Active issues" value={viewmodel.activeIssues.length} />
          <DashboardMetricCard icon={ActivityIcon} label="Running now" value={viewmodel.runningRuns} />
          <DashboardMetricCard icon={BotIcon} label="Team online" value={viewmodel.teamSize} />
        </div>

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <DashboardMetricCard helper={viewmodel.throughputDeltaLabel} label="Runs captured" value={viewmodel.runs.length} />
          <DashboardMetricCard helper={`${viewmodel.completionRate}% of tracked issues are done`} label="Completion rate" value={`${viewmodel.completionRate}%`} />
          <DashboardMetricCard helper={`${viewmodel.issueStatusBreakdown.find((item) => item.label === 'Blocked')?.value ?? 0} blocked need attention`} label="Issues in flight" value={viewmodel.activeIssues.length} />
          <DashboardMetricCard helper={`${viewmodel.completedIssues} shipped across all projects`} label="Completed issues" value={viewmodel.completedIssues} />
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.95fr)]">
          <DashboardActivityChart points={viewmodel.activity} />
          <DashboardWorkloadCard items={viewmodel.issueStatusBreakdown} priorityItems={viewmodel.priorityBreakdown} />
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(0,1fr)]">
          <DashboardProjectHealth items={viewmodel.projectHealth} />
          <Card>
            <CardHeader>
              <CardTitle>Recent runs</CardTitle>
              <CardDescription>The latest execution trail across the workspace.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {viewmodel.recentRuns.length === 0 ? (
                <p className="text-sm text-muted-foreground">No runs yet. Trigger one from the runs page or assign an issue.</p>
              ) : null}
              {viewmodel.recentRuns.map((run) => (
                <RunSummaryCard key={run.id} latestActivity={appViewmodel.runs.latestActivity(run.id)} run={run} />
              ))}
            </CardContent>
          </Card>
        </div>
      </div>
    </Page>
  )
})