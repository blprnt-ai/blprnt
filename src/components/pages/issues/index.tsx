import { useNavigate } from '@tanstack/react-router'
import { PenLineIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { IssueForm } from '@/components/forms/issue'
import { IssueFormViewmodel } from '@/components/forms/issue/issue-form.viewmodel'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { KanbanBoard } from '@/components/organisms/kanban'
import { Button } from '@/components/ui/button'
import { AppModel } from '@/models/app.model'
import { IssuesViewModel } from './issues.viewmodel'

export const IssuesPage = () => {
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

  useEffect(() => {
    if (!employeeId) return

    const viewmodel = new IssuesViewModel(employeeId)
    viewmodel.init().then(() => setViewmodel(viewmodel))
    return () => {
      viewmodel.disconnect()
    }
  }, [employeeId])

  if (!viewmodel) return <AppLoader />

  return (
    <Page className="overflow-y-auto pb-4">
      <div className="min-w-0 space-y-4">
        <div className="flex justify-end px-3 md:px-5">
          <Button type="button" variant="secondary" onClick={issueFormViewmodel.open}>
            <PenLineIcon />
            Add issue
          </Button>
        </div>
        <KanbanBoard
          employees={viewmodel.employees}
          issues={viewmodel.issues}
          onUpdateIssue={(id, data) => viewmodel.updateIssueStatus(id, data.status as IssueStatus)}
        />
      </div>
      <IssueForm viewmodel={issueFormViewmodel} />
    </Page>
  )
}
