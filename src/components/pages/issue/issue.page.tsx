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
      <div className="flex gap-3 flex-col lg:flex-row lg:justify-between">
        <div className="flex min-w-0 flex-col gap-3 max-w-5xl">
          <IssueDetails />

          <IssueHistory />
        </div>

        <div className="w-full lg:w-[240px] shrink-0">
          <IssueMetadata />
        </div>
      </div>
    </Page>
  )
}
