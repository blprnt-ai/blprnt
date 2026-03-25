import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import { employeesApi } from '@/lib/api/employees'
import { apiClient } from '@/lib/api/fetch'

export class AppModel {
  private _isOnboarded = false
  public isLoading = true
  private owner: Employee | null = null

  public static instance = new AppModel()

  private constructor() {
    makeAutoObservable(this)
  }

  public async init() {
    this._isOnboarded = localStorage.getItem('isOnboarded') === 'true'
    const ownerId = localStorage.getItem('ownerId')
    if (ownerId) apiClient.setEmployeeId(ownerId)

    const owner = await employeesApi.me()
    this.setOwner(owner)
  }

  get isOnboarded() {
    return this.owner !== null && this._isOnboarded && !this.isLoading
  }

  public setOwner(owner: Employee | null) {
    this.owner = owner
    this.isLoading = false

    if (owner) localStorage.setItem('ownerId', owner.id)
  }
}
