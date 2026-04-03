import { ActivityIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatAction, formatDate } from '../utils'
import { EmployeeNameLink } from './employee-name-link'
import { EmptyState } from './empty-state'

export const IssueActivity = observer(() => {
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
            <article key={action.id || action.createdAt.toISOString()} className="flex gap-3 p-2">
              <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <ActivityIcon className="size-4" />
              </div>
              <div className="min-w-0 flex-1 flex justify-between items-center">
                <p className="text-sm font-medium">
                  <EmployeeNameLink
                    className="transition-colors hover:text-primary hover:underline"
                    employeeId={action.creator}
                    fallback="System"
                  />
                  <span> - </span>
                  <span className="text-muted-foreground/60 font-light">{formatAction(action.action)}</span>
                </p>
                <p className="text-sm text-muted-foreground/60 font-light">{formatDate(action.createdAt)}</p>
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
})
