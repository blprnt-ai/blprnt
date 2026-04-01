import { reaction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { useIssueViewmodel } from '../issue.viewmodel'

export const IssueTitle = observer(() => {
  const viewmodel = useIssueViewmodel()

  const [isEditingTitle, setIsEditingTitle] = useState(false)
  const [titleDraft, setTitleDraft] = useState('')

  // biome-ignore lint/correctness/useExhaustiveDependencies: mobx reaction
  useEffect(() => {
    const dispose = reaction(
      () => viewmodel.issue,
      (issue) => {
        if (!issue) return
        setTitleDraft(issue.title)
        setIsEditingTitle(false)
      },
    )

    return () => dispose()
  }, [])

  const { issue } = viewmodel
  if (!issue) return null

  const handleSaveTitle = async () => {
    const nextTitle = titleDraft.trim()
    if (nextTitle.length === 0) return

    const savedIssue = await viewmodel.saveTitle(nextTitle)
    if (!savedIssue) return

    setTitleDraft(savedIssue.title)
    setIsEditingTitle(false)
  }

  const handleCancelTitle = () => {
    setTitleDraft(issue.title)
    setIsEditingTitle(false)
  }

  return (
    <>
      {isEditingTitle ? (
        <div className="space-y-3">
          <Input
            autoFocus
            className="h-11 text-xl font-medium"
            placeholder="Issue title"
            value={titleDraft}
            onChange={(event) => setTitleDraft(event.target.value)}
          />
          <div className="flex items-center justify-end gap-2">
            <Button size="sm" variant="ghost" onClick={handleCancelTitle}>
              Cancel
            </Button>
            <Button
              disabled={titleDraft.trim().length === 0 || viewmodel.isSavingTitle}
              size="sm"
              onClick={() => void handleSaveTitle()}
            >
              {viewmodel.isSavingTitle ? 'Saving...' : 'Save'}
            </Button>
          </div>
        </div>
      ) : (
        <button
          className="w-full rounded-md p-2 text-left transition-colors hover:bg-muted/60 focus-visible:bg-muted/30 focus-visible:outline-none duration-300"
          type="button"
          onClick={() => setIsEditingTitle(true)}
        >
          <CardTitle className="text-2xl">{issue.title || 'Untitled issue'}</CardTitle>
        </button>
      )}
    </>
  )
})
