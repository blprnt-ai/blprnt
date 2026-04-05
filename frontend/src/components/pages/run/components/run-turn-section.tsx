import { MessageSquareTextIcon } from 'lucide-react'
import { Fragment } from 'react/jsx-runtime'
import { Card, CardContent, CardDescription, CardHeader } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import type { TurnModel } from '@/models/turn.model'
import { getToolResultLookup, getToolUses, getTurnSummary } from '../run.presenter'
import { RunStepCard } from './run-step-card'
import { RunUsageSummary } from './run-usage-summary'

export const RunTurnSection = ({ turn, turnIndex }: { turn: TurnModel; turnIndex: number }) => {
  const summary = getTurnSummary(turn, turnIndex)
  const toolResults = getToolResultLookup(turn)
  const toolUseIds = new Set(
    turn.steps.flatMap((step) => getToolUses(step.response.contents).map((toolUse) => toolUse.tool_use_id)),
  )

  return (
    <section className="relative pl-8">
      <div className="absolute bottom-0 left-3 top-0 w-px bg-border/70" />
      <div className="absolute left-0 top-6 flex size-6 items-center justify-center rounded-full border border-border/70 bg-background shadow-sm">
        <MessageSquareTextIcon className="size-3.5 text-muted-foreground" />
      </div>

      <Card className="py-0">
        <CardHeader className="border-b border-border/60 py-5">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="space-y-1">
              <p className="text-sm font-medium">{summary.label}</p>
              <CardDescription>{summary.createdAtLabel}</CardDescription>
            </div>
            <div className="flex flex-col items-end gap-2">
              <span className="rounded-full border border-border/60 bg-background px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                {summary.stepCount} step{summary.stepCount === 1 ? '' : 's'}
              </span>
              <RunUsageSummary compact usage={turn.usage} />
            </div>
          </div>
        </CardHeader>

        <CardContent className="px-0 py-0">
          {turn.steps.map((step, stepIndex) => (
            <Fragment key={`${turn.id}-${stepIndex}`}>
              <RunStepCard
                key={`${turn.id}-${stepIndex}`}
                matchedToolResults={toolResults}
                step={step}
                toolUseIds={toolUseIds}
              />
              {stepIndex < turn.steps.length - 1 ? <Separator /> : null}
            </Fragment>
          ))}
        </CardContent>
      </Card>
    </section>
  )
}
