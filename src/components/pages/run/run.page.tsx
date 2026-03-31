import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { getRunFailureMessage } from '@/lib/runs'
import type { RunPageViewmodel } from './run.viewmodel'
import { RunHeader } from './components/run-header'
import { RunTurnSection } from './components/run-turn-section'

interface RunPageProps {
  viewmodel: RunPageViewmodel
}

export const RunPage = observer(({ viewmodel }: RunPageProps) => {
  const run = viewmodel.run
  const failureMessage = run ? getRunFailureMessage(run.status) : null

  if (!run) {
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
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <RunHeader
          canCancel={viewmodel.canCancel}
          isCancelling={viewmodel.isCancelling}
          run={run}
          onCancel={() => {
            if (!window.confirm(`Cancel run ${run.id.slice(0, 8)}?`)) return
            void viewmodel.cancel()
          }}
        />

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

          {run.turns.map((turn, turnIndex) => (
            <RunTurnSection key={turn.id} turn={turn} turnIndex={turnIndex} />
          ))}
        </div>
      </div>
    </Page>
  )
})
