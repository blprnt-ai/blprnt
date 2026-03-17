import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatDate, getInitials, resolveEmployeeName } from '../utils'
import { EmptyState } from './empty-state'

export const IssueComments = () => {
  const viewmodel = useIssueViewmodel()

  const { issue } = viewmodel
  if (!issue) return null

  return (
    <div className="space-y-3">
      {issue.comments.length > 0 ? (
        issue.comments
          .slice()
          .reverse()
          .map((comment) => (
            <article
              key={comment.id || comment.createdAt.toISOString()}
              className="rounded-sm border border-border/60 p-4"
            >
              <div className="flex items-start gap-3">
                <Avatar>
                  <AvatarFallback>{getInitials(resolveEmployeeName(comment.creator, 'You'))}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1 space-y-2">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="font-medium">{resolveEmployeeName(comment.creator, 'You')}</div>
                    <div className="text-xs text-muted-foreground">{formatDate(comment.createdAt)}</div>
                  </div>
                  <p className="whitespace-pre-wrap text-sm leading-6 text-foreground/90">{comment.comment}</p>
                </div>
              </div>
            </article>
          ))
      ) : (
        <EmptyState
          description="Start the conversation by adding a comment, a decision, or a blocker."
          title="No comments yet"
        />
      )}
    </div>
  )
}
