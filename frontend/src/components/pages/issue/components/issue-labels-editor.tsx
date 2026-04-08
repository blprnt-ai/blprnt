import { useMemo, useState } from 'react'
import { PlusIcon, XIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/ui/button'
import type { ColorVariant } from '@/components/ui/colors'
import { Input } from '@/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { ColoredSpan } from '@/components/ui/colors'
import { useIssueViewmodel } from '../issue.viewmodel'
import { IssueBadge } from './issue-badge'

export const IssueLabelsEditor = observer(({ triggerOnly = false }: { triggerOnly?: boolean }) => {
  const viewmodel = useIssueViewmodel()
  const [open, setOpen] = useState(false)

  const issue = viewmodel.issue
  const availableLabels = viewmodel.availableLabels

  const suggestedLabels = useMemo(() => {
    if (!issue) return []
    return availableLabels.filter(
      (label) =>
        !issue.labels.some((current) => current.name.toLowerCase() === label.name.toLowerCase()) &&
        label.name.toLowerCase().includes(viewmodel.labelDraft.trim().toLowerCase()),
    )
  }, [availableLabels, issue, viewmodel.labelDraft])

  if (!issue) return null

  const canQuickAdd = viewmodel.labelDraft.trim().length > 0
  const trigger = (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asDiv>
        <Button size="sm" variant="outline">
          <PlusIcon className="size-4" />
          Add label
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-72 p-3">
        <div className="space-y-3">
          <Input placeholder="Find or create label" value={viewmodel.labelDraft} onChange={(e) => viewmodel.setLabelDraft(e.target.value)} />
          <div className="max-h-48 space-y-1 overflow-y-auto">
            {suggestedLabels.map((label) => (
              <button
                key={label.name}
                className="flex w-full items-center justify-between rounded-sm px-2 py-1.5 text-left text-sm hover:bg-muted"
                type="button"
                onClick={() => {
                  void viewmodel.addLabel(label.name, label.color)
                  setOpen(false)
                }}
              >
                <span>{label.name}</span>
                <ColoredSpan className="size-2 rounded-full" color={label.color as ColorVariant} />
              </button>
            ))}
          </div>
          {canQuickAdd ? (
            <Button
              className="w-full"
              size="sm"
              onClick={() => {
                 void viewmodel.addLabel(viewmodel.labelDraft, viewmodel.nextLabelColor)
                setOpen(false)
              }}
            >
              Create “{viewmodel.labelDraft.trim()}”
            </Button>
          ) : null}
        </div>
      </PopoverContent>
    </Popover>
  )

  if (triggerOnly) return trigger

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2">
        {issue.labels.map((label) => (
          <IssueBadge
            key={label.name}
            className="inline-flex items-center gap-1 border-transparent"
          >
            <ColoredSpan className="inline-block size-2 rounded-full" color={label.color as ColorVariant} />
            {label.name}
            <button type="button" onClick={() => void viewmodel.removeLabel(label.name)}>
              <XIcon className="size-3" />
            </button>
          </IssueBadge>
        ))}
      </div>

      {issue.labels.length === 0 ? <div className="text-sm text-muted-foreground">No labels yet.</div> : null}
    </div>
  )
})