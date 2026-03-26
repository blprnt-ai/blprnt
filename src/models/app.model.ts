import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import { apiClient } from '@/lib/api/fetch'
import { EmployeeModel } from './employee.model'

const ONBOARDING_COMPLETE_KEY = 'onboarding-complete'

export class AppModel {
  public owner: EmployeeModel | null = null
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
    apiClient.setEmployeeId(owner?.id ?? null)
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
}
