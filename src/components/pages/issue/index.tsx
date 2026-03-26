import { useParams } from '@tanstack/react-router'
import { makeAutoObservable } from 'mobx'
import { useEffect, useState } from 'react'
import type { IssueDto } from '@/bindings/IssueDto'
import { Page } from '@/components/layouts/page'
import { AppLoader } from '@/components/organisms/app-loader'
import { issuesApi } from '@/lib/api/issues'
import { IssueModel } from '@/models/issue.model'

class IssueViewmodel {
  public issue: IssueModel | null = null

  constructor(private readonly issueId: string) {
    makeAutoObservable(this)
  }

  public async init() {
    const issue = await issuesApi.get(this.issueId)
    this.setIssue(issue)
  }

  private setIssue(issue: IssueDto) {
    this.issue = new IssueModel(issue)
  }
}

export const IssuePage = () => {
  const { issueId } = useParams({ from: '/issues/$issueId/' })
  const [viewmodel, setViewmodel] = useState<IssueViewmodel | null>(null)

  useEffect(() => {
    const viewmodel = new IssueViewmodel(issueId)
    viewmodel.init().then(() => {
      setViewmodel(viewmodel)
    })
  }, [issueId])

  if (!viewmodel) return <AppLoader />

  return (
    <Page>
      <div className="flex flex-col gap-4 border rounded-md p-4 ml-1 mr-2"></div>
    </Page>
  )
}
