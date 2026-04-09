import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { IssueDto } from '@/bindings/IssueDto'
import { issuesApi } from '@/lib/api/issues'
import { runsApi } from '@/lib/api/runs'
import { AppModel } from '@/models/app.model'
import { RunSummaryModel } from '@/models/run-summary.model'

type ActivityPoint = {
  label: string
  criticalCount: number
  highCount: number
  mediumCount: number
  lowCount: number
}
type BreakdownItem = { label: string; value: number; tone: string }
type ProjectHealthItem = {
  id: string
  name: string
  totalIssues: number
  openIssues: number
  completedIssues: number
  runCount: number
}

const isCompletedIssue = (issue: IssueDto) => issue.status === 'done' || issue.status === 'archived'

export class DashboardViewmodel {
  public issues: IssueDto[] = []
  public runs: RunSummaryModel[] = []
  public isLoading = true
  public errorMessage: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const [issues, archivedIssues, runPage] = await Promise.all([
        issuesApi.list(),
        issuesApi.listArchived(),
        runsApi.list(1, 100),
      ])
      runInAction(() => {
        this.issues = [...issues, ...archivedIssues]
        this.runs = runPage.items.map((run) => new RunSummaryModel(run))
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load dashboard data.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public destroy() {}

  public get activeIssues() {
    return this.issues.filter((issue) => ['todo', 'in_progress', 'blocked'].includes(issue.status))
  }

  public get completedIssues() {
    return this.issues.filter(isCompletedIssue).length
  }

  public get runningRuns() {
    return this.runs.filter((run) => run.status === 'Running').length
  }

  public get completionRate() {
    if (this.issues.length === 0) return 0
    return Math.round((this.completedIssues / this.issues.length) * 100)
  }

  public get throughputDeltaLabel() {
    const lastSeven = this.activity.slice(-7)
    const firstHalf = lastSeven.slice(0, Math.floor(lastSeven.length / 2))
    const secondHalf = lastSeven.slice(Math.floor(lastSeven.length / 2))
    const totalCompleted = (item: ActivityPoint) =>
      item.criticalCount + item.highCount + item.mediumCount + item.lowCount
    const left = firstHalf.reduce((sum, item) => sum + totalCompleted(item), 0)
    const right = secondHalf.reduce((sum, item) => sum + totalCompleted(item), 0)
    const delta = right - left
    if (delta > 0) return `+${delta} completed issues this week`
    if (delta < 0) return `${delta} completed issues versus the prior window`
    return 'Stable completion volume this week'
  }

  public get activity(): ActivityPoint[] {
    return Array.from({ length: 7 }, (_, index) => {
      const date = new Date()
      date.setHours(0, 0, 0, 0)
      date.setDate(date.getDate() - (6 - index))
      const nextDate = new Date(date)
      nextDate.setDate(nextDate.getDate() + 1)
      const label = date.toLocaleDateString(undefined, { weekday: 'short' })

      return {
        criticalCount: this.issues.filter(
          (issue) =>
            isCompletedIssue(issue) &&
            issue.priority === 'critical' &&
            new Date(issue.updated_at) >= date &&
            new Date(issue.updated_at) < nextDate,
        ).length,
        highCount: this.issues.filter(
          (issue) =>
            isCompletedIssue(issue) &&
            issue.priority === 'high' &&
            new Date(issue.updated_at) >= date &&
            new Date(issue.updated_at) < nextDate,
        ).length,
        label,
        lowCount: this.issues.filter(
          (issue) =>
            isCompletedIssue(issue) &&
            issue.priority === 'low' &&
            new Date(issue.updated_at) >= date &&
            new Date(issue.updated_at) < nextDate,
        ).length,
        mediumCount: this.issues.filter(
          (issue) =>
            isCompletedIssue(issue) &&
            issue.priority === 'medium' &&
            new Date(issue.updated_at) >= date &&
            new Date(issue.updated_at) < nextDate,
        ).length,
      }
    })
  }

  public get issueStatusBreakdown(): BreakdownItem[] {
    const statuses = [
      ['In progress', 'in_progress', 'bg-chart-1'],
      ['Todo', 'todo', 'bg-chart-2'],
      ['Blocked', 'blocked', 'bg-chart-4'],
      ['Done', 'done', 'bg-chart-3'],
    ] as const

    return statuses.map(([label, status, tone]) => ({
      label,
      tone,
      value: this.issues.filter((issue) => issue.status === status).length,
    }))
  }

  public get priorityBreakdown(): BreakdownItem[] {
    const priorities = [
      ['Critical', 'critical', 'bg-chart-5'],
      ['High', 'high', 'bg-chart-4'],
      ['Medium', 'medium', 'bg-chart-2'],
      ['Low', 'low', 'bg-chart-1'],
    ] as const

    return priorities.map(([label, priority, tone]) => ({
      label,
      tone,
      value: this.activeIssues.filter((issue) => issue.priority === priority).length,
    }))
  }

  public get projectHealth(): ProjectHealthItem[] {
    return AppModel.instance.projects
      .map((project) => {
        const projectIssues = this.issues.filter((issue) => issue.project === project.id)
        const projectRuns = this.runs.filter((run) => {
          const trigger = run.trigger
          if (typeof trigger !== 'object' || !trigger) return false
          const issueId =
            'issue_assignment' in trigger ? trigger.issue_assignment.issue_id : trigger.issue_mention?.issue_id
          return projectIssues.some((issue) => issue.id === issueId)
        })

        return {
          completedIssues: projectIssues.filter(isCompletedIssue).length,
          id: project.id,
          name: project.name,
          openIssues: projectIssues.filter((issue) => issue.status !== 'done' && issue.status !== 'archived').length,
          runCount: projectRuns.length,
          totalIssues: projectIssues.length,
        }
      })
      .sort((left, right) => right.openIssues - left.openIssues || right.runCount - left.runCount)
      .slice(0, 4)
  }

  public get recentRuns() {
    return [...this.runs].sort((left, right) => right.createdAt.getTime() - left.createdAt.getTime()).slice(0, 8)
  }

  public get teamSize() {
    return AppModel.instance.employees.length
  }
}

export const DashboardViewmodelContext = createContext<DashboardViewmodel | null>(null)

export const useDashboardViewmodel = () => {
  const viewmodel = useContext(DashboardViewmodelContext)
  if (!viewmodel) throw new Error('DashboardViewmodel not found')
  return viewmodel
}
