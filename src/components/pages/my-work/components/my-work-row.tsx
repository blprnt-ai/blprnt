import { Link } from '@tanstack/react-router'
import { ChevronRightIcon } from 'lucide-react'
import type { MyWorkItemDto } from '@/bindings/MyWorkItemDto'
import { PriorityIcon } from '@/components/molecules/priority-icon'
import { IssueBadge } from '@/components/pages/issue/components/issue-badge'
import { Card, CardContent } from '@/components/ui/card'
import { formatDate, formatLabel } from '@/components/pages/issue/utils'

interface MyWorkRowProps {
  item: MyWorkItemDto
}

export const MyWorkRow = ({ item }: MyWorkRowProps) => {
  const commentHash = item.comment_id ? `comment-${item.comment_id}` : undefined

  return (
    <Link hash={commentHash} params={{ issueId: item.issue_id }} to="/issues/$issueId">
      <Card className="border-border/60 py-0 transition-colors hover:bg-muted/35">
        <CardContent className="flex items-start justify-between gap-4 px-5 py-4">
          <div className="min-w-0 flex-1 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <IssueBadge className="font-semibold">{item.issue_identifier}</IssueBadge>
              <IssueBadge>{formatLabel(item.reason)}</IssueBadge>
              <IssueBadge>{formatLabel(item.status)}</IssueBadge>
              <IssueBadge className="inline-flex items-center gap-1.5">
                <PriorityIcon priority={item.priority} />
                {formatLabel(item.priority)}
              </IssueBadge>
            </div>

            <div className="space-y-1">
              <div className="font-medium text-foreground">{item.title}</div>
              <div className="text-sm text-muted-foreground">
                {item.project_name ?? 'No project'} · Active {formatDate(new Date(item.relevant_at))}
              </div>
            </div>

            {item.reason === 'mentioned' && item.comment_snippet ? (
              <p className="line-clamp-2 text-sm text-muted-foreground">“{item.comment_snippet}”</p>
            ) : null}
          </div>

          <ChevronRightIcon className="mt-1 size-4 shrink-0 text-muted-foreground" />
        </CardContent>
      </Card>
    </Link>
  )
}