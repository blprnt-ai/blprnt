import { useNavigate } from '@tanstack/react-router'
import { PenLineIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { IssueForm } from '@/components/forms/issue'
import { IssueFormViewmodel } from '@/components/forms/issue/issue-form.viewmodel'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { KanbanBoard } from '@/components/organisms/kanban'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { getRunIssueTarget } from '@/lib/runs'
import { AppModel } from '@/models/app.model'
import { IssuesViewModel } from './issues.viewmodel'

export const IssuesPage = observer(() => {
  const appViewmodel = useAppViewmodel()
  const employeeId = AppModel.instance.owner?.id
  const navigate = useNavigate()
  const [viewmodel, setViewmodel] = useState<IssuesViewModel | null>(null)
  const [issueFormViewmodel] = useState(
    () =>
      new IssueFormViewmodel(async (issue) => {
        await navigate({
          params: { issueId: issue.id },
          to: '/issues/$issueId',
        })
      }),
  )

  const handleNavigateToArchived = () => {
    navigate({
      to: '/issues/archived',
    })
  }

  useEffect(() => {
    if (!employeeId) return

    const viewmodel = new IssuesViewModel(employeeId)
    viewmodel.init().then(() => setViewmodel(viewmodel))
    return () => {
      viewmodel.disconnect()
    }
  }, [employeeId])

  if (!viewmodel) return <AppLoader />

  const runningIssueIds = new Set(
    appViewmodel.runs.runningRuns
      .map((run) => getRunIssueTarget(run.trigger)?.issueId)
      .filter((issueId): issueId is string => Boolean(issueId)),
  )

  return (
    <Page className="overflow-y-auto pb-4">
      <div className="min-w-0 space-y-4">
        <div className="flex flex-wrap justify-end gap-2 px-3 md:px-5">
          {viewmodel.hasSelection ? (
            <>
              <Button
                disabled={viewmodel.isArchivingSelected}
                type="button"
                variant="outline"
                onClick={() => viewmodel.clearSelection()}
              >
                Cancel selection
              </Button>
              <Button
                disabled={viewmodel.isArchivingSelected}
                type="button"
                variant="destructive-outline"
                onClick={() => void viewmodel.archiveSelectedIssues()}
              >
                Archive Selected ({viewmodel.selectedIssueIds.size})
              </Button>
            </>
          ) : null}
          <Select
            value={viewmodel.selectedLabel}
            onValueChange={(value) => void viewmodel.setSelectedLabel(value ?? '')}
          >
            <SelectTrigger className="mr-2 w-[220px]" size="sm">
              <SelectValue placeholder="Filter by label">{viewmodel.selectedLabel || 'All labels'}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All labels</SelectItem>
              {viewmodel.availableLabels.map((label) => (
                <SelectItem key={label} value={label}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="outline" onClick={handleNavigateToArchived}>
            View archived
          </Button>
          <Button variant="secondary" onClick={issueFormViewmodel.open}>
            <PenLineIcon />
            Add issue
          </Button>
        </div>
        <KanbanBoard
          employees={viewmodel.employees}
          issues={viewmodel.issues}
          liveIssueIds={runningIssueIds}
          selectedIssueIds={viewmodel.selectedIssueIds}
          onToggleIssueSelection={(issueId) => viewmodel.toggleIssueSelection(issueId)}
          onUpdateIssue={(id, status) => viewmodel.updateIssueStatus(id, status as IssueStatus)}
        />
      </div>
      <IssueForm viewmodel={issueFormViewmodel} />
    </Page>
  )
})
