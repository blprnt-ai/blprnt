import { useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { IssuePage } from './issue.page'
import { IssueViewmodel, IssueViewmodelContext } from './issue.viewmodel'

export const IssueProvider = () => {
  const { issueId } = useParams({ from: '/issues/$issueId/' })
  const [viewmodel, setViewmodel] = useState<IssueViewmodel | null>(null)

  useEffect(() => {
    const viewmodel = new IssueViewmodel(issueId)

    viewmodel.init().then(() => {
      setViewmodel(viewmodel)
    })
  }, [issueId])

  if (!viewmodel || viewmodel.isLoading) return <AppLoader />

  return (
    <IssueViewmodelContext.Provider value={viewmodel}>
      <IssuePage />
    </IssueViewmodelContext.Provider>
  )
}
