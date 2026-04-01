import { ChevronDownIcon, WrenchIcon } from 'lucide-react'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import type { StepModel } from '@/models/step.model'
import {
  formatToolId,
  getTextContents,
  getThinkingContents,
  getToolResults,
  getToolUses,
  stringifyJson,
  summarizeToolInput,
  summarizeToolResult,
} from '../run.presenter'

interface RunStepCardProps {
  matchedToolResults: Map<string, ReturnType<typeof getToolResults>[number][]>
  step: StepModel
  toolUseIds: Set<string>
}

export const RunStepCard = ({ matchedToolResults, step, toolUseIds }: RunStepCardProps) => {
  const requestTexts = getTextContents(step.request.contents)
  const responseTexts = getTextContents(step.response.contents)
  const thinkings = getThinkingContents(step.response.contents)
  const toolUses = getToolUses(step.response.contents)
  const stepToolResults = getToolResults(step.request.contents)
  const unmatchedToolResults = stepToolResults.filter((result) => !toolUseIds.has(result.tool_use_id))

  return (
    <div className="space-y-5 py-5">
      {requestTexts.length > 0 ? <RunTextSection label="Request" texts={requestTexts} /> : null}

      {thinkings.length > 0 ? (
        <Collapsible>
          <CollapsibleTrigger className="flex w-full items-center justify-between rounded-sm border border-border/60 bg-background/60 px-3 py-2 text-left text-xs uppercase tracking-[0.18em] text-muted-foreground transition-colors hover:bg-muted/40">
            Reasoning
            <ChevronDownIcon className="size-4" />
          </CollapsibleTrigger>
          <CollapsibleContent className="pt-3">
            <RunTextSection muted texts={thinkings} />
          </CollapsibleContent>
        </Collapsible>
      ) : null}

      {toolUses.length > 0 ? (
        <RunToolSection
          items={toolUses.map((toolUse) => ({
            body: summarizeToolInput(toolUse.input),
            input: stringifyJson(toolUse.input),
            key: toolUse.tool_use_id,
            results: matchedToolResults.get(toolUse.tool_use_id) ?? [],
            title: formatToolId(toolUse.tool_id),
          }))}
        />
      ) : null}

      {responseTexts.length > 0 ? <RunTextSection label="Response" texts={responseTexts} /> : null}

      {unmatchedToolResults.length > 0 ? (
        <RunToolResultList
          label="Results"
          results={unmatchedToolResults.map((result) => ({
            body: summarizeToolResult(result.content),
            key: result.tool_use_id,
            title: formatToolId(result.tool_id),
            value: stringifyJson(result.content),
          }))}
        />
      ) : null}
    </div>
  )
}

const RunTextSection = ({ label, muted = false, texts }: { label?: string; muted?: boolean; texts: string[] }) => {
  return (
    <section className="space-y-3">
      {label ? <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">{label}</p> : null}
      <div className="space-y-3">
        {texts.map((text, index) => (
          <div
            key={`${label ?? 'text'}-${index}`}
            className={cn('rounded-sm border border-border/60 p-4', muted ? 'bg-muted/25' : 'bg-background/80')}
          >
            <pre className="whitespace-pre-wrap break-words font-sans text-sm leading-6">{text}</pre>
          </div>
        ))}
      </div>
    </section>
  )
}

const RunToolSection = ({
  items,
}: {
  items: Array<{
    body: string
    input: string
    key: string
    results: ReturnType<typeof getToolResults>[number][]
    title: string
  }>
}) => {
  return (
    <section className="space-y-3">
      <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Tool calls</p>
      <div className="space-y-3">
        {items.map((item) => (
          <div key={item.key} className="rounded-sm border border-border/60 bg-background/80 p-4">
            <div className="flex items-start gap-3">
              <div className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <WrenchIcon className="size-4" />
              </div>
              <div className="min-w-0 flex-1 space-y-3">
                <div>
                  <p className="font-medium">{item.title}</p>
                  <p className="text-sm text-muted-foreground">{item.body}</p>
                </div>
                <RunRawDetails label="Input" value={item.input} />
                {item.results.length > 0 ? (
                  <>
                    <Separator />
                    <RunToolResultList
                      label="Result"
                      results={item.results.map((result, index) => ({
                        body: summarizeToolResult(result.content),
                        key: `${item.key}-${index}`,
                        value: stringifyJson(result.content),
                      }))}
                    />
                  </>
                ) : null}
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}

const RunToolResultList = ({
  label,
  results,
}: {
  label: string
  results: Array<{
    body: string
    key: string
    value: string
  }>
}) => {
  return (
    <section className="space-y-3">
      <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">{label}</p>
      <div className="space-y-3">
        {results.map((result) => (
          <div key={result.key} className="rounded-sm border border-border/60 bg-muted/20 p-4">
            <RunRawDetails label={result.body} value={result.value} />
          </div>
        ))}
      </div>
    </section>
  )
}

const RunRawDetails = ({ label, value }: { label: string; value: string }) => {
  return (
    <Collapsible>
      <CollapsibleTrigger className="flex w-full items-center justify-between text-left text-xs uppercase tracking-[0.18em] text-muted-foreground">
        {label}
        <ChevronDownIcon className="size-4" />
      </CollapsibleTrigger>
      <CollapsibleContent className="pt-3">
        <pre className="overflow-x-auto rounded-sm border border-border/60 bg-muted/30 p-3 text-xs leading-5 text-muted-foreground">
          {value}
        </pre>
      </CollapsibleContent>
    </Collapsible>
  )
}
