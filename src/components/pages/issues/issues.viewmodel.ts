import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { Employee } from '@/bindings/Employee'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { employeesApi } from '@/lib/api/employees'
import { issuesApi } from '@/lib/api/issues'
import { connectIssueStream } from '@/lib/api/issues-stream'
import { AppModel } from '@/models/app.model'

export class IssuesViewModel {
  public issues: IssueDto[] = []
  public employees: Employee[] = []
  private readonly employeeId: string
  private socket: WebSocket | null = null

  constructor(employeeId: string) {
    this.employeeId = employeeId
    makeAutoObservable(this)
  }

  public async init() {
    const issues = await issuesApi.list()
    this.setIssues(issues)

    const employees = await employeesApi.list()
    this.setEmployees(employees)
    this.connect()
  }

  private setIssues = (issues: IssueDto[]) => {
    this.issues = issues
  }

  private setEmployees = (employees: Employee[]) => {
    this.employees = employees
    AppModel.instance.setEmployees(employees)
  }

  private connect() {
    this.disconnect()
    this.socket = connectIssueStream(this.employeeId, {
      onMessage: (message) => {
        runInAction(() => {
          if (message.type === 'snapshot') {
            this.setIssues(message.snapshot.issues)
            return
          }

          const existingIndex = this.issues.findIndex((issue) => issue.id === message.issue.id)
          if (existingIndex === -1) {
            this.issues = [...this.issues, message.issue]
            return
          }

          this.issues = this.issues.map((issue) => (issue.id === message.issue.id ? message.issue : issue))
        })
      },
    })
  }

  public disconnect() {
    if (this.socket) this.socket.close()
    this.socket = null
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
