import { ArrowUpIcon } from 'lucide-react'
import type * as React from 'react'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import {
  DEFAULT_REASONING_OPTION,
  formatDefaultReasoningLabel,
  formatReasoningEffort,
  reasoningEffortOptions,
} from '@/lib/reasoning'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import type { RunPageViewmodel } from '../run.viewmodel'

interface RunComposerProps {
  viewmodel: RunPageViewmodel
}

export const RunComposer = ({ viewmodel }: RunComposerProps) => {
  const submitLabel = viewmodel.isDraft ? 'Start conversation' : 'Send message'

  return (
    <div className="fixed right-3 bottom-3 left-3 z-30 md:right-5 md:bottom-4 md:left-[calc(var(--sidebar-width)+1rem)]">
      <form
        className="mx-auto w-full max-w-3xl"
        onSubmit={(event) => {
          event.preventDefault()
          void viewmodel.sendMessage()
        }}
      >
        <div className="mb-2 flex justify-start">
          <Select
            value={viewmodel.composerReasoningEffort ?? DEFAULT_REASONING_OPTION}
            onValueChange={(value) =>
              viewmodel.setComposerReasoningEffort(
                value === DEFAULT_REASONING_OPTION ? null : (value as ReasoningEffort),
              )
            }
          >
            <SelectTrigger className="h-9 w-auto min-w-40 rounded-full border-border/70 bg-background/92 px-3 text-xs shadow-sm">
              <SelectValue>
                {viewmodel.composerReasoningEffort
                  ? formatReasoningEffort(viewmodel.composerReasoningEffort)
                  : formatDefaultReasoningLabel(viewmodel.employeeReasoningEffort)}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={DEFAULT_REASONING_OPTION}>
                {formatDefaultReasoningLabel(viewmodel.employeeReasoningEffort)}
              </SelectItem>
              {reasoningEffortOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {formatReasoningEffort(option.value)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="relative">
          <Textarea
            className="min-h-12 rounded-2xl border-border/70 bg-background/96 pr-14 py-3 shadow-[0_10px_30px_-18px_rgba(15,23,42,0.45)] backdrop-blur"
            maxRows={6}
            minRows={2}
            placeholder={viewmodel.composerPlaceholder}
            value={viewmodel.composerValue}
            onChange={(event) => viewmodel.setComposerValue(event.target.value)}
            onKeyDown={(event: React.KeyboardEvent<HTMLTextAreaElement>) => {
              if (event.key === 'Enter' && !event.shiftKey && (event.metaKey || event.ctrlKey)) {
                event.preventDefault()
                void viewmodel.sendMessage()
              }
            }}
          />
          <Button
            aria-label={submitLabel}
            className="absolute right-2.5 bottom-2.5 rounded-full"
            disabled={!viewmodel.canSendMessage}
            size="icon-sm"
            type="submit"
          >
            <ArrowUpIcon className="size-4" />
          </Button>
        </div>
      </form>
    </div>
  )
}
