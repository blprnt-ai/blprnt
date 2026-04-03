import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import type { IssueCommentModel } from '@/models/issue-comment.model'
import { formatDate, getInitials, resolveEmployeeName } from '../utils'
import { IssueCommentBody } from './issue-comment-body'

interface IssueCommentCardProps {
  comment: IssueCommentModel
}

export const IssueCommentCard = ({ comment }: IssueCommentCardProps) => {
  return (
    <article className="rounded-sm border border-border/60 p-4">
      <div className="flex items-start gap-3">
        <Avatar>
          <AvatarFallback>{getInitials(resolveEmployeeName(comment.creator, 'You'))}</AvatarFallback>
        </Avatar>

        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="font-medium">{resolveEmployeeName(comment.creator, 'You')}</div>
            <div className="text-xs text-muted-foreground">{formatDate(comment.createdAt)}</div>
          </div>
          <div className="text-sm leading-6 text-foreground/90">
            <IssueCommentBody comment={comment} />
          </div>
        </div>
      </div>
    </article>
  )
}