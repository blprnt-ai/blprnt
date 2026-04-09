import { Link } from '@tanstack/react-router'
import { ArrowUpRightIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useRef, useState } from 'react'
import { Page } from '@/components/layouts/page'
import { ConfirmationDialog } from '@/components/molecules/confirmation-dialog'
import { ScrollToBottomButton } from '@/components/molecules/scroll-to-bottom-button'
import { buttonVariants } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useScrollAnchor } from '@/hooks/use-scroll-anchor'
import { getRunFailureMessage } from '@/lib/runs'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import { RunComposer } from './components/run-composer'
import { RunDraftHeader } from './components/run-draft-header'
import { RunHeader } from './components/run-header'
import { RunTurnSection } from './components/run-turn-section'
import type { RunPageViewmodel } from './run.viewmodel'

interface RunPageProps {
  viewmodel: RunPageViewmodel
}

export const RunPage = observer(({ viewmodel }: RunPageProps) => {
  const [isCancelDialogOpen, setIsCancelDialogOpen] = useState(false)
  const pageRef = useRef<HTMLDivElement | null>(null)
  const scrollAnchor = useScrollAnchor()
  const run = viewmodel.run
  const failureMessage = run ? getRunFailureMessage(run.status) : null
  const associatedIssueTarget = viewmodel.associatedIssueTarget
  const employeeName = viewmodel.employeeId
    ? (AppModel.instance.resolveEmployeeName(viewmodel.employeeId) ?? 'Unknown employee')
    : 'Unknown employee'

  useEffect(() => {
    if (!run) return
    if (!scrollAnchor.isNearBottom) return

    scrollAnchor.scrollToBottom('smooth')
  }, [run, scrollAnchor])

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
    <>
      <Page
        ref={(element) => {
          pageRef.current = element
          scrollAnchor.setContainer(element)
        }}
        className="overflow-y-auto px-3 pb-28 md:px-5 md:pb-32"
      >
        <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
          {run ? (
            <RunHeader
              canCancel={viewmodel.canCancel}
              isCancelling={viewmodel.isCancelling}
              run={run}
              onCancel={() => setIsCancelDialogOpen(true)}
            />
          ) : (
            <RunDraftHeader employeeName={employeeName} />
          )}

          <div className="min-w-0 space-y-4">
            {associatedIssueTarget ? (
              <Card>
                <CardContent className="flex items-center justify-between gap-3 py-4">
                  <div className="min-w-0">
                    <p className="text-sm font-medium text-foreground">Associated issue</p>
                    <p className="text-sm text-muted-foreground">
                      {associatedIssueTarget.commentHash
                        ? 'Open the mentioned issue and jump to the comment.'
                        : 'Open the linked issue.'}
                    </p>
                  </div>
                  <Link
                    className={cn(buttonVariants({ size: 'sm', variant: 'outline' }), 'shrink-0')}
                    hash={associatedIssueTarget.commentHash}
                    params={{ issueId: associatedIssueTarget.issueId }}
                    to="/issues/$issueId"
                  >
                    Open issue
                    <ArrowUpRightIcon className="size-4" />
                  </Link>
                </CardContent>
              </Card>
            ) : null}

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

            {run?.turns.map((turn, turnIndex) => (
              <RunTurnSection key={turn.id} turn={turn} turnIndex={turnIndex} />
            ))}
          </div>

          {viewmodel.showComposer ? <RunComposer viewmodel={viewmodel} /> : null}
        </div>
        {run ? (
          <ConfirmationDialog
            cancelLabel="Keep running"
            confirmLabel="Cancel run"
            description={`Run ${run.id.slice(0, 8)} will be stopped immediately.`}
            onConfirm={() => void viewmodel.cancel()}
            onOpenChange={setIsCancelDialogOpen}
            open={isCancelDialogOpen}
            title="Cancel this run?"
          />
        ) : null}
      </Page>
      <ScrollToBottomButton
        className={viewmodel.showComposer ? 'bottom-24 md:bottom-28' : undefined}
        onClick={() => scrollAnchor.scrollToBottom()}
        visible={!scrollAnchor.isNearBottom}
      />
    </>
  )
})
