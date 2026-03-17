import { Card, CardContent } from '@/components/ui/card'
import { useIssueViewmodel } from '../issue.viewmodel'
import { formatLabel } from '../utils'
import { IssueBadge } from './issue-badge'
import { IssueDescription } from './issue-description'
import { IssueTitle } from './issue-title'

export const IssueDetails = () => {
  const viewmodel = useIssueViewmodel()

  const { issue } = viewmodel
  if (!issue) return null

  return (
    <Card>
      <CardContent className="flex flex-col gap-6">
        <div className="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
          <span>{issue.identifier}</span>
          <IssueBadge>{formatLabel(issue.status)}</IssueBadge>
          <IssueBadge>{formatLabel(issue.priority)}</IssueBadge>
        </div>
        <div className="flex flex-col gap-4">
          <IssueTitle />

          <IssueDescription />
        </div>
      </CardContent>
    </Card>
  )
}
