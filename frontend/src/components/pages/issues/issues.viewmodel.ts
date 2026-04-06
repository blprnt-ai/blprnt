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
  public selectedLabel = ''
  public allIssues: IssueDto[] = []
  public selectedIssueIds = new Set<string>()
  public isArchivingSelected = false
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
    this.allIssues = issues
    this.issues = this.selectedLabel
      ? issues.filter((issue) => issue.labels.some((label) => label.name === this.selectedLabel))
      : issues
    this.selectedIssueIds = new Set(
      Array.from(this.selectedIssueIds).filter((issueId) => this.allIssues.some((issue) => issue.id === issueId && issue.status !== 'archived')),
    )
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

          const existingIndex = this.allIssues.findIndex((issue) => issue.id === message.issue.id)
          if (existingIndex === -1) {
            this.setIssues([...this.allIssues, message.issue])
            return
          }

          this.setIssues(this.allIssues.map((issue) => (issue.id === message.issue.id ? message.issue : issue)))
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
    if (status === 'archived') {
      this.selectedIssueIds.delete(issueId)
    }

    try {
      await issuesApi.update(issueId, { status })
    } catch (error) {
      toast.error('Failed to update issue status')
      console.error(error)
      this.setIssues(issues)
    }
  }

  public get availableLabels() {
    return Array.from(new Set(this.allIssues.flatMap((issue) => issue.labels.map((label) => label.name)))).sort((a, b) => a.localeCompare(b))
  }

  public async setSelectedLabel(label: string) {
    this.selectedLabel = label
    runInAction(() => {
      this.issues = label
        ? this.allIssues.filter((issue) => issue.labels.some((issueLabel) => issueLabel.name === label))
        : this.allIssues
    })
  }

  public get hasSelection() {
    return this.selectedIssueIds.size > 0
  }

  public isSelected(issueId: string) {
    return this.selectedIssueIds.has(issueId)
  }

  public toggleIssueSelection(issueId: string) {
    if (this.selectedIssueIds.has(issueId)) {
      this.selectedIssueIds.delete(issueId)
      return
    }

    this.selectedIssueIds.add(issueId)
  }

  public clearSelection() {
    this.selectedIssueIds.clear()
  }

  public async archiveSelectedIssues() {
    const selectedIssueIds = Array.from(this.selectedIssueIds)
    if (selectedIssueIds.length === 0 || this.isArchivingSelected) return

    const previousIssues = [...this.allIssues]
    this.isArchivingSelected = true
    this.clearSelection()
    this.setIssues(this.allIssues.map((issue) => (selectedIssueIds.includes(issue.id) ? { ...issue, status: 'archived' } : issue)))

    try {
      await Promise.all(selectedIssueIds.map((issueId) => issuesApi.update(issueId, { status: 'archived' })))
      toast.success(`Archived ${selectedIssueIds.length} issue${selectedIssueIds.length === 1 ? '' : 's'}.`)
    } catch (error) {
      console.error(error)
      this.setIssues(previousIssues)
      toast.error('Failed to archive the selected issues')
    } finally {
      runInAction(() => {
        this.isArchivingSelected = false
      })
    }
  }
}

export const IssuesPageViewModelContext = createContext<IssuesViewModel | null>(null)
export const useIssuesPageViewModel = () => {
  const viewmodel = useContext(IssuesPageViewModelContext)
  if (!viewmodel) throw new Error('IssuesPageViewModel not found')

  return viewmodel
}
