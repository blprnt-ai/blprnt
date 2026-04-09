import { FilePlus2Icon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type * as React from 'react'
import { useId, useRef } from 'react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useIssueViewmodel } from '../issue.viewmodel'
import { getInitials } from '../utils'

export const IssueAddComment = observer(() => {
  const viewmodel = useIssueViewmodel()

  const attachmentInputRef = useRef<HTMLInputElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const reopenIssueId = useId()

  const { issue } = viewmodel
  if (!issue) return null

  const handleAttachmentChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    if (files.length === 0) return

    await viewmodel.addAttachments(files)
    event.target.value = ''
  }

  const handleMentionSelection = (employee = viewmodel.activeMentionSuggestion) => {
    if (!employee) return

    const nextCaret = viewmodel.selectCommentMention(employee)
    if (nextCaret === null) return

    requestAnimationFrame(() => {
      textareaRef.current?.focus()
      textareaRef.current?.setSelectionRange(nextCaret, nextCaret)
    })
  }

  return (
    <div>
      <form
        onSubmit={(event) => {
          event.preventDefault()
          void viewmodel.submitComment()
        }}
      >
        <div className="relative">
          {viewmodel.activeMentionQuery && viewmodel.mentionSuggestions.length > 0 ? (
            <div className="absolute inset-x-0 bottom-full z-10 mb-2 rounded-md border border-border/80 bg-popover p-1 shadow-md">
              {viewmodel.visibleMentionSuggestions.map((employee) => (
                <button
                  key={employee.id}
                  className="flex w-full items-center gap-3 rounded-sm px-3 py-2 text-left text-sm hover:bg-muted data-[active=true]:bg-muted"
                  data-active={viewmodel.activeMentionSuggestion?.id === employee.id}
                  type="button"
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => {
                    const index = viewmodel.visibleMentionSuggestions.findIndex(
                      (candidate) => candidate.id === employee.id,
                    )
                    if (index >= 0) viewmodel.activeMentionSuggestionIndex = index
                  }}
                  onClick={() => handleMentionSelection(employee)}
                >
                  <span className="flex size-7 items-center justify-center rounded-full bg-muted text-xs font-medium">
                    {getInitials(employee.name)}
                  </span>
                  <span className="min-w-0 flex-1 truncate">{employee.name}</span>
                </button>
              ))}
            </div>
          ) : null}

          <Textarea
            ref={textareaRef}
            maxRows={8}
            minRows={4}
            placeholder="Add context, decisions, or next steps..."
            value={viewmodel.commentDraft}
            onChange={(event) =>
              viewmodel.setCommentDraft(event.target.value, event.target.selectionStart ?? event.target.value.length)
            }
            onClick={(event) =>
              viewmodel.setCommentDraft(
                viewmodel.commentDraft,
                event.currentTarget.selectionStart ?? viewmodel.commentDraft.length,
              )
            }
            onKeyDown={(event) => {
              if (!viewmodel.activeMentionQuery || viewmodel.visibleMentionSuggestions.length === 0) return

              if (event.key === 'ArrowDown') {
                event.preventDefault()
                viewmodel.moveActiveMentionSelection(1)
                return
              }

              if (event.key === 'ArrowUp') {
                event.preventDefault()
                viewmodel.moveActiveMentionSelection(-1)
                return
              }

              if (event.key === 'Enter') {
                event.preventDefault()
                handleMentionSelection()
              }
            }}
            onKeyUp={(event) =>
              viewmodel.setCommentDraft(
                viewmodel.commentDraft,
                event.currentTarget.selectionStart ?? viewmodel.commentDraft.length,
              )
            }
          />
        </div>

        {issue.status === 'done' ? (
          <label className="mt-4 flex items-end justify-end gap-2 text-sm" htmlFor={reopenIssueId}>
            <span>Reopen issue</span>
            <input
              checked={viewmodel.reopenIssueOnComment}
              className="size-4"
              id={reopenIssueId}
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
