import { ActivityIcon, BotIcon, CircleDashedIcon, SparklesIcon } from 'lucide-react'
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
        <section className="relative overflow-hidden rounded-3xl border border-border/60 bg-[radial-gradient(circle_at_top_left,rgba(34,211,238,0.22),transparent_38%),radial-gradient(circle_at_top_right,rgba(168,85,247,0.16),transparent_32%),linear-gradient(135deg,rgba(15,23,42,0.96),rgba(12,18,32,0.88))] px-6 py-6 text-white shadow-2xl shadow-cyan-950/10 md:px-8 md:py-8">
          <div className="absolute inset-y-0 right-0 w-1/3 bg-[linear-gradient(180deg,transparent,rgba(255,255,255,0.05),transparent)] blur-3xl" />
          <div className="relative flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
            <div className="max-w-2xl space-y-3">
              <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.28em] text-cyan-100/80">
                <SparklesIcon className="size-4" />
                Ops pulse
              </div>
              <div className="space-y-2">
                <h1 className="text-3xl font-semibold tracking-tight md:text-4xl">A clearer picture of how blprnt is moving.</h1>
                <p className="max-w-xl text-sm text-slate-200/85 md:text-base">
                  Live run throughput, issue pressure, and project momentum in one place.
                </p>
              </div>
            </div>
            <div className="grid gap-3 sm:grid-cols-3 xl:min-w-[420px] xl:max-w-xl">
              <DashboardMetricCard icon={CircleDashedIcon} label="Active issues" tone="dark" value={viewmodel.activeIssues.length} />
              <DashboardMetricCard icon={ActivityIcon} label="Running now" tone="dark" value={viewmodel.runningRuns} />
              <DashboardMetricCard icon={BotIcon} label="Team online" tone="dark" value={viewmodel.teamSize} />
            </div>
          </div>
        </section>

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