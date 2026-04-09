import { observer } from 'mobx-react-lite'
import { useDashboardViewmodel } from '../dashboard.viewmodel'
import { DashboardMetricCard } from './dashboard-metric-card'

export const DashboardMetrics = observer(() => {
  const viewmodel = useDashboardViewmodel()

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <DashboardMetricCard
        helper={viewmodel.throughputDeltaLabel}
        label="Runs captured"
        value={viewmodel.runs.length}
      />
      <DashboardMetricCard
        helper={`${viewmodel.completionRate}% of tracked issues are done`}
        label="Completion rate"
        value={`${viewmodel.completionRate}%`}
      />
      <DashboardMetricCard
        helper={`${viewmodel.issueStatusBreakdown.find((item) => item.label === 'Blocked')?.value ?? 0} blocked need attention`}
        label="Issues in flight"
        value={viewmodel.activeIssues.length}
      />
      <DashboardMetricCard
        helper={`${viewmodel.completedIssues} shipped across all projects`}
        label="Completed issues"
        value={viewmodel.completedIssues}
      />
    </div>
  )
})
