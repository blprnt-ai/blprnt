import { reaction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { MarkdownEditor, MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { Button } from '@/components/ui/button'
import { restoreDoubleLineBreaks, restoreSingleLineBreaks } from '@/lib/line-breaks'
import { cn } from '@/lib/utils'
import { useIssueViewmodel } from '../issue.viewmodel'

export const IssueDescription = observer(() => {
  const viewmodel = useIssueViewmodel()

  const [isEditingDescription, setIsEditingDescription] = useState(false)
  const [descriptionDraft, setDescriptionDraft] = useState(restoreDoubleLineBreaks(viewmodel.issue?.description ?? ''))

  // biome-ignore lint/correctness/useExhaustiveDependencies: mobx reaction
  useEffect(() => {
    const dispose = reaction(
      () => viewmodel.issue,
      (issue) => {
        if (!issue) return
        setDescriptionDraft(restoreDoubleLineBreaks(issue.description))
        setIsEditingDescription(false)
      },
    )

    return () => dispose()
  }, [])

  const { issue } = viewmodel
  if (!issue) return null

  const handleSaveDescription = async () => {
    const nextDescription = descriptionDraft.trim()
    if (nextDescription.length === 0) return

    const savedIssue = await viewmodel.saveDescription(restoreSingleLineBreaks(descriptionDraft))
    if (!savedIssue) return

    setDescriptionDraft(restoreDoubleLineBreaks(savedIssue.description))
    setIsEditingDescription(false)
  }

  const handleCancelDescription = () => {
    setDescriptionDraft(restoreDoubleLineBreaks(issue.description))
    setIsEditingDescription(false)
  }

  return (
    <>
      {isEditingDescription ? (
        <div className="flex flex-col gap-4">
          <MarkdownEditor
            placeholder="Describe the issue, context, and expected outcome..."
            value={descriptionDraft}
            onChange={setDescriptionDraft}
          />
          <div className="flex items-center justify-end gap-2">
            <Button size="sm" variant="ghost" onClick={handleCancelDescription}>
              Cancel
            </Button>
            <Button
              disabled={descriptionDraft.trim().length === 0 || viewmodel.isSavingDescription}
              size="sm"
              onClick={() => void handleSaveDescription()}
            >
              {viewmodel.isSavingDescription ? 'Saving...' : 'Save'}
            </Button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          className={cn(
            'w-full rounded-md text-left transition-colors hover:bg-muted/60 focus-visible:bg-muted/30 focus-visible:outline-none duration-300',
          )}
          onClick={() => setIsEditingDescription(true)}
        >
          <MarkdownEditorPreview
            value={restoreDoubleLineBreaks(issue.description) || 'No description has been added yet.'}
          />
        </button>
      )}
    </>
  )
})
