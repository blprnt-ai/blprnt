import { Link } from '@tanstack/react-router'
import { ArchiveIcon, RefreshCwIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { statusBadge, statusBadgeDefault } from '@/lib/status-colors'
import { cn } from '@/lib/utils'
import { AppModel } from '@/models/app.model'
import { IssueBadge } from '../issue/components/issue-badge'
import { formatDate } from '../issue/utils'
import { ArchivedIssuesViewmodel } from './issues-archived.viewmodel'

export const ArchivedIssuesPage = observer(() => {
  const [viewmodel] = useState(() => new ArchivedIssuesViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) {
    return <AppLoader />
  }

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ArchiveIcon className="size-4" />
              Archived issues
            </CardTitle>
            <CardDescription>Read-only archive, sorted by newest created first.</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap items-center justify-between gap-2 text-sm text-muted-foreground">
            <span>{viewmodel.issues.length} archived issue{viewmodel.issues.length === 1 ? '' : 's'}</span>
            <div className="flex items-center gap-2">
              <Button type="button" variant="outline" onClick={() => void viewmodel.init()}>
                <RefreshCwIcon />
                Refresh
              </Button>
              <Button render={<Link to="/issues" />} variant="outline">
                Back to issues
              </Button>
            </div>
          </CardContent>
        </Card>

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        {viewmodel.issues.length === 0 ? (
          <Card>
            <CardContent className="py-6 text-sm text-muted-foreground">No archived issues yet.</CardContent>
          </Card>
        ) : null}

        {viewmodel.issues.map((issue) => (
          <Card key={issue.id} className="transition-colors hover:border-primary/30">
            <CardContent className="py-5">
              <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                <div className="min-w-0 space-y-2">
                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <Link className="font-mono text-foreground hover:underline" params={{ issueId: issue.id }} to="/issues/$issueId">
                      {issue.identifier}
                    </Link>
                    <span className={cn('rounded-full px-2 py-0.5 text-xs font-medium', statusBadge.archived ?? statusBadgeDefault)}>
                      Archived
                    </span>
                    <span>Created {formatDate(new Date(issue.created_at))}</span>
                  </div>
                  <div className="space-y-1">
                    <Link className="block text-base font-medium hover:underline" params={{ issueId: issue.id }} to="/issues/$issueId">
                      {issue.title}
                    </Link>
                    {issue.description ? <p className="line-clamp-2 text-sm text-muted-foreground">{issue.description}</p> : null}
                  </div>
                  {issue.labels.length > 0 ? (
                    <div className="flex flex-wrap gap-1.5">
                      {issue.labels.map((label) => (
                        <IssueBadge key={label.name}>{label.name}</IssueBadge>
                      ))}
                    </div>
                  ) : null}
                </div>
                <div className="flex shrink-0 flex-col gap-1 text-sm text-muted-foreground md:items-end">
                  <span>{AppModel.instance.resolveProjectName(issue.project) ?? 'No project'}</span>
                  <span>{AppModel.instance.resolveEmployeeName(issue.assignee) ?? 'Unassigned'}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </Page>
  )
})