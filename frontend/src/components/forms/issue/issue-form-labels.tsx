import { PlusIcon, XIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useMemo } from 'react'
import { IssueBadge } from '@/components/pages/issue/components/issue-badge'
import { Button } from '@/components/ui/button'
import type { ColorVariant } from '@/components/ui/colors'
import { ColoredSpan } from '@/components/ui/colors'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { IssueFormViewmodel } from './issue-form.viewmodel'

interface IssueFormLabelsProps {
  viewmodel: IssueFormViewmodel
}

export const IssueFormLabels = observer(({ viewmodel }: IssueFormLabelsProps) => {
  const suggestedLabels = useMemo(() => {
    return viewmodel.availableLabels.filter(
      (label) =>
        !viewmodel.issue.labels.some((current) => current.name.toLowerCase() === label.name.toLowerCase()) &&
        label.name.toLowerCase().includes(viewmodel.labelDraft.trim().toLowerCase()),
    )
  }, [viewmodel])

  const canQuickAdd = viewmodel.labelDraft.trim().length > 0

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-3">
        <Label>Labels</Label>
        <Popover>
          <PopoverTrigger asDiv>
            <Button size="sm" type="button" variant="outline">
              <PlusIcon className="size-4" />
              Add label
            </Button>
          </PopoverTrigger>
          <PopoverContent align="end" className="w-72 p-3">
            <div className="space-y-3">
              <Input
                placeholder="Find or create label"
                value={viewmodel.labelDraft}
                onChange={(event) => viewmodel.setLabelDraft(event.target.value)}
              />

              <div className="max-h-48 space-y-1 overflow-y-auto">
                {suggestedLabels.map((label) => (
                  <button
                    key={label.name}
                    className="flex w-full items-center justify-between rounded-sm px-2 py-1.5 text-left text-sm hover:bg-muted"
                    type="button"
                    onClick={() => viewmodel.addLabel(label.name, label.color)}
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
                  type="button"
                  onClick={() => viewmodel.addLabel(viewmodel.labelDraft, viewmodel.nextLabelColor)}
                >
                  Create “{viewmodel.labelDraft.trim()}”
                </Button>
              ) : null}
            </div>
          </PopoverContent>
        </Popover>
      </div>

      <div className="flex min-h-9 flex-wrap gap-2 rounded-md border border-border/60 bg-muted/20 p-2">
        {viewmodel.issue.labels.length > 0 ? (
          viewmodel.issue.labels.map((label) => (
            <IssueBadge key={label.name} className="inline-flex items-center gap-1 border-transparent text-white">
              <ColoredSpan className="inline-block size-2 rounded-full" color={label.color as ColorVariant} />
              {label.name}
              <button type="button" onClick={() => viewmodel.removeLabel(label.name)}>
                <XIcon className="size-3" />
              </button>
            </IssueBadge>
          ))
        ) : (
          <div className="text-sm text-muted-foreground">No labels yet.</div>
        )}
      </div>
    </div>
  )
})
