import { makeAutoObservable, reaction } from 'mobx'
import { createContext, useContext } from 'react'
import type { BootstrapOwnerPayload } from './bindings/BootstrapOwnerPayload'
import type { Employee } from './bindings/Employee'
import type { LoginPayload } from './bindings/LoginPayload'
import { authApi } from './lib/api/auth'
import { employeesApi } from './lib/api/employees'
import { apiClient } from './lib/api/fetch'
import { EmployeesViewmodel } from './employees.viewmodel'
import { issuesApi } from './lib/api/issues'
import { projectsApi } from './lib/api/projects'
import { AppModel } from './models/app.model'
import { RunsViewmodel } from './runs.viewmodel'

export class AppViewmodel {
  public employees = new EmployeesViewmodel()
  public runs = new RunsViewmodel()

  constructor() {
    makeAutoObservable(this)
    apiClient.setUnauthorizedHandler(this.handleUnauthorized)
    reaction(
      () => AppModel.instance.owner?.id ?? null,
      (ownerId) => {
        if (!ownerId) {
          this.employees.disconnect()
          this.runs.disconnect()
          return
        }

        this.employees.connect(ownerId)
        this.runs.connect(ownerId)
      },
      { fireImmediately: true },
    )
  }

  public init = async () => {
    AppModel.instance.setAuthStatus('loading')

    try {
      const employee = await employeesApi.me()
      if (!employee) return
      await this.hydrateAuthenticatedApp(employee)
      return
    } catch {
    }

    const authStatus = await authApi.status()
    AppModel.instance.clearSession()
    AppModel.instance.setOwnerExists(authStatus.has_owner)
    AppModel.instance.setOwnerLoginConfigured(authStatus.owner_login_configured)
  }

  public async login(payload: LoginPayload) {
    const employee = await authApi.login(payload)
    await this.hydrateAuthenticatedApp(employee)
  }

  public async bootstrapOwner(payload: BootstrapOwnerPayload) {
    const employee = await authApi.bootstrapOwner(payload)
    await this.hydrateAuthenticatedApp(employee)
  }

  public async logout() {
    try {
      await authApi.logout()
    } finally {
      this.handleUnauthorized()
    }
  }

  public get isOnboarded() {
    return AppModel.instance.isOnboarded
  }

  public get hasOwner() {
    return AppModel.instance.hasOwner
  }

  public get isOwnerLoginConfigured() {
    return AppModel.instance.isOwnerLoginConfigured
  }

  public get isAuthenticated() {
    return AppModel.instance.isAuthenticated
  }

  public get isAuthResolved() {
    return AppModel.instance.isAuthResolved
  }

  public setIsOnboarded(isOnboarded: boolean) {
    AppModel.instance.setIsOnboarded(isOnboarded)
  }

  private async hydrateAuthenticatedApp(employee: Employee) {
    AppModel.instance.setOwner(employee)
    const [employees, projects, issues] = await Promise.all([employeesApi.list(), projectsApi.list(), issuesApi.list()])
    AppModel.instance.setEmployees(employees)
    AppModel.instance.setIssues(issues)
    AppModel.instance.setProjects(projects)
    AppModel.instance.setIsOnboarded(issues.length > 0)
  }

  private handleUnauthorized = () => {
    AppModel.instance.clearSession()
    window.dispatchEvent(new CustomEvent('blprnt:unauthorized'))
  }
}

export const AppViewmodelContext = createContext<AppViewmodel | null>(null)
export const useAppViewmodel = () => {
  const appViewmodel = useContext(AppViewmodelContext)
  if (!appViewmodel) throw new Error('AppViewmodel not found')

  return appViewmodel
}
