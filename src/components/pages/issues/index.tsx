import { useEffect, useState } from 'react'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { KanbanBoard } from '@/components/organisms/kanban'
import { IssuesViewModel } from './issues.viewmodel'

export const IssuesPage = () => {
  const [viewmodel, setViewmodel] = useState<IssuesViewModel | null>(null)

  useEffect(() => {
    const viewmodel = new IssuesViewModel()
    viewmodel.init().then(() => setViewmodel(viewmodel))
  }, [])

  if (!viewmodel) return <AppLoader />

  return (
    <Page>
      <KanbanBoard
        employees={viewmodel.employees}
        issues={viewmodel.issues}
        onUpdateIssue={(id, data) => viewmodel.updateIssueStatus(id, data.status as IssueStatus)}
      />
    </Page>
  )
}
