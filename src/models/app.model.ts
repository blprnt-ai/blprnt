import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import { employeesApi } from '@/lib/api/employees'

export class AppModel {
  public isLoading = true
  private owner: Employee | null = null

  constructor() {
    makeAutoObservable(this)
  }

  public async init() {
    const owner = await employeesApi.me()
    this.setOwner(owner)
  }

  get isOnboarded() {
    return this.owner !== null && !this.isLoading
  }

  private setOwner(owner: Employee | null) {
    this.owner = owner
    this.isLoading = false
  }
}
