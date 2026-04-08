import { Link } from '@tanstack/react-router'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { useNow } from '@/hooks/use-now'
import { formatRelativeTime } from '@/lib/time'
import type { IssueCommentModel } from '@/models/issue-comment.model'
import { formatDate, getInitials, resolveEmployeeName } from '../utils'
import { EmployeeNameLink } from './employee-name-link'
import { IssueCommentBody } from './issue-comment-body'

interface IssueCommentCardProps {
  comment: IssueCommentModel
}

export const IssueCommentCard = ({ comment }: IssueCommentCardProps) => {
  const now = useNow()
  const runLabel = comment.runId ? `Run ${comment.runId.slice(0, 8)}` : null

  return (
    <article className="scroll-mt-4 rounded-sm border border-border/60 p-4" id={comment.id ? `comment-${comment.id}` : undefined}>
      <div className="flex items-start gap-3">
        <Avatar>
          <AvatarFallback>{getInitials(resolveEmployeeName(comment.creator, 'You'))}</AvatarFallback>
        </Avatar>

        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2 text-sm">
              <EmployeeNameLink
                className="font-medium text-foreground transition-colors hover:text-primary hover:underline"
                employeeId={comment.creator}
                fallback="You"
              />
              {runLabel ? (
                <>
                  <span className="text-muted-foreground">-</span>
                  <Link
                    className="text-muted-foreground transition-colors hover:text-primary hover:underline"
                    params={{ runId: comment.runId }}
                    to="/runs/$runId"
                  >
                    {runLabel}
                  </Link>
                </>
              ) : null}
            </div>
            <div className="text-xs text-muted-foreground" title={formatDate(comment.createdAt)}>
              {formatRelativeTime(comment.createdAt, now)}
            </div>
          </div>
          <div className="text-sm leading-6 text-foreground/90">
            <IssueCommentBody comment={comment} />
          </div>
        </div>
      </div>
    </article>
  )
}