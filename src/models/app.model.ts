import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import { apiClient } from '@/lib/api/fetch'

export class AppModel {
  public owner: Employee | null = null
  public hasProvider = false
  public hasProjects = false
  public hasCeo = false
  public hasIssues = false

  public static instance = new AppModel()

  private constructor() {
    makeAutoObservable(this)
  }

  public get isOnboarded() {
    return this.owner !== null && this.hasProvider && this.hasProjects && this.hasIssues
  }

  public setOwner(owner: Employee) {
    this.owner = owner
    apiClient.setEmployeeId(owner?.id ?? null)
  }

  public setHasProvider(hasProvider: boolean) {
    this.hasProvider = hasProvider
  }

  public setHasProjects(hasProjects: boolean) {
    this.hasProjects = hasProjects
  }

  public setHasCeo(hasCeo: boolean) {
    this.hasCeo = hasCeo
  }

  public setHasIssues(hasIssues: boolean) {
    this.hasIssues = hasIssues
  }
}
