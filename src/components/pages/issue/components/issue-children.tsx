import { Link } from '@tanstack/react-router'
import { ChevronRightIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatLabel, resolveEmployeeName } from '../utils'
import { EmptyState } from './empty-state'
import { IssueBadge } from './issue-badge'

export const IssueChildren = observer(() => {
  const viewmodel = useIssueViewmodel()

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">Child issues</h3>
        </div>
        <IssueBadge>{viewmodel.childIssues.length} total</IssueBadge>
      </div>

      {viewmodel.isLoadingChildIssues && (
        <EmptyState description="Loading linked child issues..." title="Fetching child issues" />
      )}
      {viewmodel.childIssues.length > 0 && (
        <div className="flex flex-col gap-2">
          {viewmodel.childIssues.map((childIssue) => (
            <Link
              key={childIssue.id || childIssue.identifier}
              params={{ issueId: childIssue.id! }}
              to="/issues/$issueId"
            >
              <div
                key={childIssue.id || childIssue.identifier}
                className="rounded-sm border border-border/60 p-4 transition-colors hover:bg-muted/20"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      <span>{childIssue.identifier || childIssue.id || 'Child issue'}</span>
                      <IssueBadge>{formatLabel(childIssue.status)}</IssueBadge>
                      <IssueBadge>{formatLabel(childIssue.priority)}</IssueBadge>
                    </div>
                    <div className="mt-2 text-sm font-medium">{childIssue.title || 'Untitled child issue'}</div>
                    <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                      {childIssue.description || 'No description yet.'}
                    </p>
                  </div>
                  <div className="flex items-center gap-1 text-xs text-muted-foreground">
                    <span>{resolveEmployeeName(childIssue.assignee, 'Unassigned')}</span>
                    <ChevronRightIcon className="size-4" />
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
      {viewmodel.childIssues.length === 0 && (
        <EmptyState
          description="Child issues will appear here once this issue is broken into smaller tasks."
          title="No child issues yet"
        />
      )}
    </div>
  )
})
