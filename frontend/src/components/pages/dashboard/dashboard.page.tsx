import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { DashboardActivityChart } from './components/dashboard-activity-chart'
import { DashboardRuns } from './components/dashboard-runs'
import { DashboardWorkloadCard } from './components/dashboard-workload-card'
import { useDashboardViewmodel } from './dashboard.viewmodel'

export const DashboardPage = observer(() => {
  const viewmodel = useDashboardViewmodel()

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <DashboardRuns />

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.95fr)]">
          <DashboardActivityChart points={viewmodel.activity} />
          <DashboardWorkloadCard items={viewmodel.issueStatusBreakdown} priorityItems={viewmodel.priorityBreakdown} />
        </div>
      </div>
    </Page>
  )
})
