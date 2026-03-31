import { FilePlus2Icon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type * as React from 'react'
import { useRef } from 'react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useIssueViewmodel } from '../issue.viewmodel'

export const IssueAddComment = observer(() => {
  const viewmodel = useIssueViewmodel()

  const attachmentInputRef = useRef<HTMLInputElement>(null)

  const { issue } = viewmodel
  if (!issue) return null

  const handleAttachmentChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    if (files.length === 0) return

    await viewmodel.addAttachments(files)
    event.target.value = ''
  }

  return (
    <div>
      <form
        onSubmit={(event) => {
          event.preventDefault()
          void viewmodel.submitComment()
        }}
      >
        <Textarea
          maxRows={8}
          minRows={4}
          placeholder="Add context, decisions, or next steps..."
          value={viewmodel.commentDraft}
          onChange={(event) => viewmodel.setCommentDraft(event.target.value)}
        />

        {issue.status === 'done' ? (
          <label className="mt-4 flex items-end justify-end gap-2 text-sm" htmlFor="reopen-issue-on-comment">
            <span>Reopen issue</span>
            <input
              checked={viewmodel.reopenIssueOnComment}
              className="size-4"
              id="reopen-issue-on-comment"
              type="checkbox"
              onChange={(event) => viewmodel.setReopenIssueOnComment(event.target.checked)}
            />
          </label>
        ) : null}

        <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <input
              ref={attachmentInputRef}
              multiple
              className="hidden"
              type="file"
              onChange={(event) => {
                void handleAttachmentChange(event)
              }}
            />
            <Button type="button" variant="outline" onClick={() => attachmentInputRef.current?.click()}>
              <FilePlus2Icon className="size-4" />
              {viewmodel.isUploadingAttachments ? 'Uploading...' : 'Add attachment'}
            </Button>
            <span className="text-xs text-muted-foreground">Upload screenshots, specs, logs, or related files.</span>
          </div>

          <Button disabled={!viewmodel.canSubmitComment} type="submit">
            {viewmodel.isSubmittingComment ? 'Posting...' : 'Post comment'}
          </Button>
        </div>
      </form>
    </div>
  )
})
