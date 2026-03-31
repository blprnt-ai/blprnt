import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { getRunFailureMessage } from '@/lib/runs'
import { AppModel } from '@/models/app.model'
import type { RunPageViewmodel } from './run.viewmodel'
import { RunComposer } from './components/run-composer'
import { RunDraftHeader } from './components/run-draft-header'
import { RunHeader } from './components/run-header'
import { RunTurnSection } from './components/run-turn-section'

interface RunPageProps {
  viewmodel: RunPageViewmodel
}

export const RunPage = observer(({ viewmodel }: RunPageProps) => {
  const run = viewmodel.run
  const failureMessage = run ? getRunFailureMessage(run.status) : null
  const employeeName = viewmodel.employeeId
    ? (AppModel.instance.resolveEmployeeName(viewmodel.employeeId) ?? 'Unknown employee')
    : 'Unknown employee'

  if (!run && !viewmodel.isDraft) {
    return (
      <Page className="overflow-y-auto px-3 pb-6 md:px-5">
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">
            {viewmodel.errorMessage ?? 'Run not found.'}
          </CardContent>
        </Card>
      </Page>
    )
  }

  return (
    <Page className="overflow-y-auto px-3 pb-28 md:px-5 md:pb-32">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        {run ? (
          <RunHeader
            canCancel={viewmodel.canCancel}
            isCancelling={viewmodel.isCancelling}
            run={run}
            onCancel={() => {
              if (!window.confirm(`Cancel run ${run.id.slice(0, 8)}?`)) return
              void viewmodel.cancel()
            }}
          />
        ) : (
          <RunDraftHeader employeeName={employeeName} />
        )}

        <div className="min-w-0 space-y-4">
          {failureMessage ? (
            <Card className="border-destructive/30 bg-destructive/5 py-0">
              <CardContent className="py-4 text-sm text-destructive">{failureMessage}</CardContent>
            </Card>
          ) : null}

          {viewmodel.errorMessage ? (
            <Card>
              <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
            </Card>
          ) : null}

          {run?.turns.map((turn, turnIndex) => <RunTurnSection key={turn.id} turn={turn} turnIndex={turnIndex} />)}
        </div>

        {viewmodel.showComposer ? <RunComposer viewmodel={viewmodel} /> : null}
      </div>
    </Page>
  )
})
