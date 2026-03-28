import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { apiClient } from '@/lib/api/fetch'
import { EmployeeModel } from './employee.model'

const ONBOARDING_COMPLETE_KEY = 'onboarding-complete'

export class AppModel {
  public owner: EmployeeModel | null = null
  public employees: Employee[] = []
  public projects: ProjectDto[] = []
  public _isOnboarded = false

  public static instance = new AppModel()

  private constructor() {
    makeAutoObservable(this)
  }

  public get hasOwner() {
    return this.owner !== null
  }

  public setOwner(owner: Employee) {
    this.owner = new EmployeeModel(owner)
    this.upsertEmployee(owner)
    apiClient.setEmployeeId(owner?.id ?? null)
  }

  public setEmployees(employees: Employee[]) {
    this.employees = sortEmployees(employees)
  }

  public setProjects(projects: ProjectDto[]) {
    this.projects = projects
  }

  public upsertEmployee(employee: Employee) {
    const index = this.employees.findIndex((candidate) => candidate.id === employee.id)

    if (index === -1) {
      this.employees = sortEmployees([...this.employees, employee])
      return
    }

    this.employees = sortEmployees(
      this.employees.map((candidate) => (candidate.id === employee.id ? employee : candidate)),
    )
  }

  public upsertProject(project: ProjectDto) {
    const index = this.projects.findIndex((candidate) => candidate.id === project.id)

    if (index === -1) {
      this.projects = [...this.projects, project]
      return
    }

    this.projects = this.projects.map((candidate) => (candidate.id === project.id ? project : candidate))
  }

  public resolveEmployeeName(employeeId: string | null | undefined) {
    if (!employeeId) return null
    const employee = this.employees.find((employee) => employee.id === employeeId)
    if (employee?.role === 'owner') return 'You'

    return employee?.name ?? employeeId
  }

  public resolveProjectName(projectId: string | null | undefined) {
    if (!projectId) return null

    return this.projects.find((project) => project.id === projectId)?.name ?? projectId
  }

  public get isOnboarded() {
    if (this._isOnboarded) return true
    this._isOnboarded = localStorage.getItem(ONBOARDING_COMPLETE_KEY) === 'true'

    return this._isOnboarded
  }

  public setIsOnboarded(isOnboarded: boolean) {
    localStorage.setItem(ONBOARDING_COMPLETE_KEY, isOnboarded.toString())
    this._isOnboarded = isOnboarded
  }

  public resetAfterDatabaseNuke() {
    this.owner = null
    this.employees = []
    this.projects = []
    apiClient.setEmployeeId(null)
    this.setIsOnboarded(false)
  }
}

const sortEmployees = (employees: Employee[]) => {
  return [...employees].sort((left, right) => {
    if (left.role === 'owner' && right.role !== 'owner') return -1
    if (left.role !== 'owner' && right.role === 'owner') return 1

    return left.name.localeCompare(right.name)
  })
}
