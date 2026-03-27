import { ActivityIcon } from 'lucide-react'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatAction, formatDate, resolveEmployeeName } from '../utils'
import { EmptyState } from './empty-state'

export const IssueActivity = () => {
  const viewmodel = useIssueViewmodel()

  const { issue } = viewmodel
  if (!issue) return null

  return (
    <div className="space-y-3">
      {issue.actions.length > 0 ? (
        issue.actions
          .slice()
          .reverse()
          .map((action) => (
            <article
              key={action.id || action.createdAt.toISOString()}
              className="flex gap-3 rounded-sm border border-border/60 p-4"
            >
              <div className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <ActivityIcon className="size-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium">{formatAction(action.action)}</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  {resolveEmployeeName(action.creator, 'System')} · {formatDate(action.createdAt)}
                </p>
              </div>
            </article>
          ))
      ) : (
        <EmptyState
          description="Actions like status updates, assignments, and uploads will appear here."
          title="No activity yet"
        />
      )}
    </div>
  )
}
