import { Page } from '@/components/layouts/page'
import { IssueDetails } from './components/issue-details'
import { IssueHistory } from './components/issue-history'
import { IssueMetadata } from './components/issue-metadata'
import { IssueNotFound } from './components/issue-not-found'
import { useIssueViewmodel } from './issue.viewmodel'

export const IssuePage = () => {
  const viewmodel = useIssueViewmodel()

  if (!viewmodel.issue) return <IssueNotFound />

  return (
    <Page className="p-1 pr-2 overflow-y-auto">
      <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="flex min-w-0 flex-col gap-3">
          <IssueDetails />

          <IssueHistory />
        </div>

        <IssueMetadata />
      </div>
    </Page>
  )
}
