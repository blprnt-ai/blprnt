import { useParams } from '@tanstack/react-router'
import { reaction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { AppModel } from '@/models/app.model'
import { HeaderBreadcrumbModel } from '@/models/header-breadcrumb.model'
import { IssuePage } from './issue.page'
import { IssueViewmodel, IssueViewmodelContext } from './issue.viewmodel'

const ISSUE_ROUTE_ID = '/issues/$issueId/'

export const IssueProvider = observer(() => {
  const { issueId } = useParams({ from: '/issues/$issueId/' })
  const employeeId = AppModel.instance.owner?.id
  const [viewmodel, setViewmodel] = useState<IssueViewmodel | null>(null)

  useEffect(() => {
    if (!employeeId) return

    const viewmodel = new IssueViewmodel(issueId, employeeId)

    viewmodel.init().then(() => {
      setViewmodel(viewmodel)
    })

    return () => {
      viewmodel.disconnect()
    }
  }, [employeeId, issueId])

  useEffect(() => {
    if (!viewmodel) return

    const dispose = reaction(
      () => viewmodel.issue?.title,
      () => {
        if (!viewmodel.issue) {
          HeaderBreadcrumbModel.instance.clearLabel(ISSUE_ROUTE_ID)
          return
        }

        const title = viewmodel.issue.title.trim()
        HeaderBreadcrumbModel.instance.setLabel(ISSUE_ROUTE_ID, title || 'Untitled issue')
      },
      {
        fireImmediately: true,
      },
    )

    return () => {
      dispose()
      HeaderBreadcrumbModel.instance.clearLabel(ISSUE_ROUTE_ID)
    }
  }, [viewmodel])

  if (!viewmodel || viewmodel.isLoading) return <AppLoader />

  return (
    <IssueViewmodelContext.Provider value={viewmodel}>
      <IssuePage />
    </IssueViewmodelContext.Provider>
  )
})
