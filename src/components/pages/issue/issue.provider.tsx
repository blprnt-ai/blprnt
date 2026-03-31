import { useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { AppModel } from '@/models/app.model'
import { IssuePage } from './issue.page'
import { IssueViewmodel, IssueViewmodelContext } from './issue.viewmodel'

export const IssueProvider = () => {
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

  if (!viewmodel || viewmodel.isLoading) return <AppLoader />

  return (
    <IssueViewmodelContext.Provider value={viewmodel}>
      <IssuePage />
    </IssueViewmodelContext.Provider>
  )
}
