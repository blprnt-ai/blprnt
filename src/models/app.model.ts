import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { apiClient } from '@/lib/api/fetch'
import { EmployeeModel } from './employee.model'

export type AuthStatus = 'loading' | 'authenticated' | 'unauthenticated'

export class AppModel {
  public owner: EmployeeModel | null = null
  public employees: Employee[] = []
  public projects: ProjectDto[] = []
  public _isOnboarded = false
  public ownerExists = true
  public ownerLoginConfigured = true
  public authStatus: AuthStatus = 'loading'
  private removedEmployeeIds = new Set<string>()

  public static instance = new AppModel()

  private constructor() {
    makeAutoObservable(this)
  }

  public get hasOwner() {
    return this.ownerExists
  }

  public get isAuthenticated() {
    return this.authStatus === 'authenticated' && this.owner !== null
  }

  public get isOwnerLoginConfigured() {
    return this.ownerLoginConfigured
  }

  public get isAuthResolved() {
    return this.authStatus !== 'loading'
  }

  public setOwner(owner: Employee) {
    this.owner = new EmployeeModel(owner)
    this.ownerExists = true
    this.ownerLoginConfigured = true
    this.authStatus = 'authenticated'
    apiClient.setEmployeeId(owner.id)
    this.removedEmployeeIds.delete(owner.id)
    this.upsertEmployee(owner)
  }

  public setOwnerExists(ownerExists: boolean) {
    this.ownerExists = ownerExists
    if (!ownerExists) this.ownerLoginConfigured = false
  }

  public setOwnerLoginConfigured(ownerLoginConfigured: boolean) {
    this.ownerLoginConfigured = ownerLoginConfigured
  }

  public setAuthStatus(authStatus: AuthStatus) {
    this.authStatus = authStatus
  }

  public setEmployees(employees: Employee[]) {
    this.employees = sortEmployees(employees)
    for (const employee of employees) {
      this.removedEmployeeIds.delete(employee.id)
    }
  }

  public setProjects(projects: ProjectDto[]) {
    this.projects = projects
  }

  public upsertEmployee(employee: Employee) {
    this.removedEmployeeIds.delete(employee.id)
    const index = this.employees.findIndex((candidate) => candidate.id === employee.id)

    if (index === -1) {
      this.employees = sortEmployees([...this.employees, employee])
      return
    }

    this.employees = sortEmployees(
      this.employees.map((candidate) => (candidate.id === employee.id ? employee : candidate)),
    )
  }

  public removeEmployee(employeeId: string) {
    this.removedEmployeeIds.add(employeeId)
    this.employees = this.employees.filter((employee) => employee.id !== employeeId)
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
    if (this.removedEmployeeIds.has(employeeId)) return null
    if (employeeId === this.owner?.id) return 'You'
    const employee = this.employees.find((employee) => employee.id === employeeId)

    return employee?.name ?? employeeId
  }

  public resolveProjectName(projectId: string | null | undefined) {
    if (!projectId) return null

    return this.projects.find((project) => project.id === projectId)?.name ?? projectId
  }

  public get isOnboarded() {
    return this._isOnboarded
  }

  public setIsOnboarded(isOnboarded: boolean) {
    this._isOnboarded = isOnboarded
  }

  public clearSession() {
    this.owner = null
    this.authStatus = 'unauthenticated'
    apiClient.setEmployeeId(null)
    this.employees = []
    this.projects = []
    this.removedEmployeeIds.clear()
    this._isOnboarded = false
  }

  public resetAfterDatabaseNuke() {
    this.clearSession()
    this.ownerExists = false
    this.ownerLoginConfigured = false
  }
}

const sortEmployees = (employees: Employee[]) => {
  return [...employees].sort((left, right) => {
    if (left.role === 'owner' && right.role !== 'owner') return -1
    if (left.role !== 'owner' && right.role === 'owner') return 1

    return left.name.localeCompare(right.name)
  })
}
