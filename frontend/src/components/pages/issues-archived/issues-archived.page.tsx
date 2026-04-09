import { Link } from '@tanstack/react-router'
import { ArchiveIcon, RefreshCwIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
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
          <CardContent className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center justify-between gap-2 text-sm text-muted-foreground">
              <span>
                {viewmodel.filteredIssues.length} of {viewmodel.issues.length} archived issue
                {viewmodel.issues.length === 1 ? '' : 's'}
              </span>
              <div className="flex items-center gap-2">
                <Button type="button" variant="outline" onClick={() => void viewmodel.init()}>
                  <RefreshCwIcon />
                  Refresh
                </Button>
                <Button render={<Link to="/issues" />} variant="outline">
                  Back to issues
                </Button>
              </div>
            </div>
            <Input
              placeholder="Filter archived issues"
              value={viewmodel.searchQuery}
              onChange={(event) => viewmodel.setSearchQuery(event.target.value)}
            />
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

        {viewmodel.issues.length > 0 && viewmodel.filteredIssues.length === 0 ? (
          <Card>
            <CardContent className="py-6 text-sm text-muted-foreground">
              No archived issues match that filter.
            </CardContent>
          </Card>
        ) : null}

        {viewmodel.filteredIssues.length > 0 ? (
          <Card className="overflow-hidden">
            <div className="divide-y">
              {viewmodel.filteredIssues.map((issue) => (
                <div key={issue.id} className="px-4 py-2 transition-colors hover:bg-muted/30">
                  <div className="flex min-w-0 items-center gap-2 text-sm leading-none">
                    <Link
                      className="shrink-0 font-mono text-xs text-foreground hover:underline"
                      params={{ issueId: issue.id }}
                      to="/issues/$issueId"
                    >
                      {issue.identifier}
                    </Link>
                    <span
                      className={cn(
                        'shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium',
                        statusBadge.archived ?? statusBadgeDefault,
                      )}
                    >
                      Archived
                    </span>
                    <Link
                      className="min-w-0 truncate font-medium hover:underline"
                      params={{ issueId: issue.id }}
                      to="/issues/$issueId"
                    >
                      {issue.title}
                    </Link>
                    {issue.labels.length > 0 ? <IssueBadge>{issue.labels[0]?.name}</IssueBadge> : null}
                    <span className="shrink-0 text-xs text-muted-foreground">•</span>
                    <span className="shrink min-w-0 truncate text-xs text-muted-foreground">
                      {AppModel.instance.resolveProjectName(issue.project) ?? 'No project'}
                    </span>
                    <span className="shrink-0 text-xs text-muted-foreground">•</span>
                    <span className="shrink min-w-0 truncate text-xs text-muted-foreground">
                      {AppModel.instance.resolveEmployeeName(issue.assignee) ?? 'Unassigned'}
                    </span>
                    <span className="shrink-0 text-xs text-muted-foreground">•</span>
                    <span className="shrink-0 text-xs text-muted-foreground">
                      {formatDate(new Date(issue.created_at))}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        ) : null}
      </div>
    </Page>
  )
})
