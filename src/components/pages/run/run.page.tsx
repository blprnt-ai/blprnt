import { ChevronDownIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { ToolId } from '@/bindings/ToolId'
import type { TurnStepContent } from '@/bindings/TurnStepContent'
import { Page } from '@/components/layouts/page'
import { RunStatusChip } from '@/components/organisms/run-status-chip'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { formatRunTime, formatRunTrigger } from '@/lib/runs'
import { AppModel } from '@/models/app.model'
import type { RunPageViewmodel } from './run.viewmodel'

interface RunPageProps {
  viewmodel: RunPageViewmodel
}

export const RunPage = observer(({ viewmodel }: RunPageProps) => {
  const run = viewmodel.run

  if (!run) {
    return (
      <Page className="overflow-y-auto p-1 pr-2">
        <Card>
          <CardContent className="py-6 text-sm text-muted-foreground">
            {viewmodel.errorMessage ?? 'Run not found.'}
          </CardContent>
        </Card>
      </Page>
    )
  }

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <Card>
          <CardHeader>
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="space-y-1">
                <CardTitle>Run {run.id.slice(0, 8)}</CardTitle>
                <CardDescription>
                  {AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'} ·{' '}
                  {formatRunTrigger(run.trigger)}
                </CardDescription>
              </div>
              {viewmodel.canCancel ? (
                <Button
                  disabled={viewmodel.isCancelling}
                  type="button"
                  variant="destructive-outline"
                  onClick={() => {
                    if (!window.confirm(`Cancel run ${run.id.slice(0, 8)}?`)) return
                    void viewmodel.cancel()
                  }}
                >
                  {viewmodel.isCancelling ? 'Cancelling...' : 'Cancel run'}
                </Button>
              ) : null}
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
              <RunStatusChip status={run.status} />
              <span>Created {formatRunTime(run.createdAt)}</span>
              <span>Started {formatRunTime(run.startedAt)}</span>
              <span>Completed {formatRunTime(run.completedAt)}</span>
            </div>
            {viewmodel.errorMessage ? <p className="text-sm text-destructive">{viewmodel.errorMessage}</p> : null}
          </CardContent>
        </Card>

        {run.turns.map((turn, turnIndex) => (
          <Card key={turn.id}>
            <CardHeader>
              <CardTitle>Turn {turnIndex + 1}</CardTitle>
              <CardDescription>{formatRunTime(turn.createdAt)}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {turn.steps.map((step, stepIndex) => (
                <div key={`${turn.id}-${stepIndex}`} className="space-y-2 rounded-sm border border-foreground/10 p-3">
                  <div className="flex items-center justify-between text-xs uppercase tracking-wide text-muted-foreground">
                    <span>{step.contents.role}</span>
                    <span>{step.status}</span>
                  </div>
                  <div className="space-y-2">
                    {step.contents.contents.map((content, index) => (
                      <RunContentBlock key={index} content={content} />
                    ))}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        ))}
      </div>
    </Page>
  )
})

const RunContentBlock = ({ content }: { content: TurnStepContent }) => {
  if ('Text' in content) {
    return <pre className="whitespace-pre-wrap text-sm">{content.Text.text}</pre>
  }

  if ('Thinking' in content) {
    return (
      <Collapsible>
        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-sm border border-foreground/10 px-3 py-2 text-left text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Thinking
          <ChevronDownIcon className="size-4" />
        </CollapsibleTrigger>
        <CollapsibleContent className="mt-2 rounded-sm border border-foreground/10 bg-muted/40 p-3">
          <pre className="whitespace-pre-wrap text-sm text-muted-foreground">{content.Thinking.thinking}</pre>
        </CollapsibleContent>
      </Collapsible>
    )
  }

  if ('ToolUse' in content) {
    return (
      <pre className="whitespace-pre-wrap text-sm text-muted-foreground">
        Tool call: {formatToolId(content.ToolUse.tool_id)}
      </pre>
    )
  }

  if ('ToolResult' in content) {
    return (
      <pre className="whitespace-pre-wrap text-sm text-muted-foreground">
        Tool result ({formatToolId(content.ToolResult.tool_id)}): {JSON.stringify(content.ToolResult.content, null, 2)}
      </pre>
    )
  }

  if ('Image64' in content) {
    return <p className="text-sm text-muted-foreground">Image content</p>
  }

  return null
}

const formatToolId = (toolId: ToolId) => {
  if (typeof toolId === 'string') return toolId
  if ('mcp' in toolId) return `mcp:${toolId.mcp}`
  if ('unknown' in toolId) return toolId.unknown
  return 'unknown'
}
