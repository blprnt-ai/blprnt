import { Link } from '@tanstack/react-router'
import { MarkdownEditorPreview } from '@/components/organisms/markdown-editor'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import type { IssueCommentModel } from '@/models/issue-comment.model'
import { segmentCommentWithMentions } from '../comment-mentions'

interface IssueCommentBodyProps {
  comment: IssueCommentModel
}

export const IssueCommentBody = ({ comment }: IssueCommentBodyProps) => {
  if (comment.mentions.length === 0) {
    return <MarkdownEditorPreview value={comment.comment} />
  }

  const segments = segmentCommentWithMentions(comment.comment, comment.mentions)

  return (
    <div className="whitespace-pre-wrap rounded-md text-sm leading-6">
      {segments.map((segment, index) => {
        if (segment.kind === 'text') {
          return <span key={`${segment.kind}-${index}`}>{segment.value}</span>
        }

        const employeeName = AppModel.instance.resolveEmployeeName(segment.employeeId) ?? segment.value.slice(1)

        if (!segment.employeeId) {
          return (
            <span key={`${segment.kind}-unknown-${index}`} className="font-medium text-primary" title={employeeName}>
              {segment.value}
            </span>
          )
        }

        return (
          <Link
            key={`${segment.kind}-${segment.employeeId}-${index}`}
            className={cn('font-medium text-primary transition-colors hover:underline')}
            params={{ employeeId: segment.employeeId }}
            title={employeeName}
            to="/employees/$employeeId"
          >
            {segment.value}
          </Link>
        )
      })}
    </div>
  )
}
