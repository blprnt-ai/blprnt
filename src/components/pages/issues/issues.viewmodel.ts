import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { Employee } from '@/bindings/Employee'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { employeesApi } from '@/lib/api/employees'
import { issuesApi } from '@/lib/api/issues'

export class IssuesViewModel {
  public issues: IssueDto[] = []
  public employees: Employee[] = []

  constructor() {
    makeAutoObservable(this)
  }

  public async init() {
    const issues = await issuesApi.list()
    this.setIssues(issues)

    const employees = await employeesApi.list()
    this.setEmployees(employees)
  }

  private setIssues = (issues: IssueDto[]) => {
    this.issues = issues
  }

  private setEmployees = (employees: Employee[]) => {
    this.employees = employees
  }

  public updateIssueStatus = async (issueId: string, status: IssueStatus) => {
    const issues = [...this.issues]
    const updatedIssues = this.issues.map((issue) => (issue.id === issueId ? { ...issue, status } : issue))
    this.setIssues(updatedIssues)

    try {
      await issuesApi.update(issueId, { status })
    } catch (error) {
      toast.error('Failed to update issue status')
      console.error(error)
      this.setIssues(issues)
    }
  }
}

export const IssuesPageViewModelContext = createContext<IssuesViewModel | null>(null)
export const useIssuesPageViewModel = () => {
  const viewmodel = useContext(IssuesPageViewModelContext)
  if (!viewmodel) throw new Error('IssuesPageViewModel not found')

  return viewmodel
}
