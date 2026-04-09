import { Link } from '@tanstack/react-router'
import { ArrowUpRightIcon, MessageSquareTextIcon } from 'lucide-react'
import type { MyWorkItemDto } from '@/bindings/MyWorkItemDto'
import { PriorityIcon } from '@/components/molecules/priority-icon'
import { IssueBadge } from '@/components/pages/issue/components/issue-badge'
import { formatDate, formatLabel } from '@/components/pages/issue/utils'
import { Card, CardContent } from '@/components/ui/card'

interface MyWorkRowProps {
  item: MyWorkItemDto
}

export const MyWorkRow = ({ item }: MyWorkRowProps) => {
  const commentHash = item.comment_id ? `comment-${item.comment_id}` : undefined
  const isMention = item.reason === 'mentioned'

  return (
    <Link hash={commentHash} params={{ issueId: item.issue_id }} to="/issues/$issueId">
      <Card className="border-border/60 bg-background/90 py-0 transition-all hover:-translate-y-0.5 hover:bg-muted/25">
        <CardContent className="flex items-start justify-between gap-4 px-5 py-5">
          <div className="min-w-0 flex-1 space-y-4">
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
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted-foreground">
                <span>{item.project_name ?? 'No project'}</span>
                <span>Active {formatDate(new Date(item.relevant_at))}</span>
              </div>
            </div>

            {isMention && item.comment_snippet ? (
              <div className="rounded-sm border border-border/60 bg-muted/20 px-3 py-3 text-sm text-muted-foreground">
                <div className="mb-2 inline-flex items-center gap-2 text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  <MessageSquareTextIcon className="size-3.5" /> Mention context
                </div>
                <p className="line-clamp-2">“{item.comment_snippet}”</p>
              </div>
            ) : null}
          </div>

          <div className="mt-1 flex size-9 shrink-0 items-center justify-center rounded-full border border-border/60 bg-muted/20 text-muted-foreground">
            <ArrowUpRightIcon className="size-4" />
          </div>
        </CardContent>
      </Card>
    </Link>
  )
}
