import { makeAutoObservable, runInAction } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import type { Provider } from '@/bindings/Provider'
import { employeesApi } from '@/lib/api/employees'
import { EmployeeModel } from '@/models/employee.model'

export class EmployeeFormViewmodel {
  public isOpen = false
  public isSaving = false
  public employee: EmployeeModel = new EmployeeModel()
  private onCreated?: (employee: Employee) => Promise<void> | void

  constructor(onCreated?: (employee: Employee) => Promise<void> | void) {
    this.onCreated = onCreated
    makeAutoObservable(this)
  }

  public get canSave() {
    return this.employee.isIdentityValid && !this.isSaving
  }

  public open = () => {
    this.reset()
    this.isOpen = true
  }

  public close = () => {
    if (this.isSaving) return
    this.isOpen = false
    this.reset()
  }

  public setOpen = (isOpen: boolean) => {
    if (isOpen) {
      this.open()
      return
    }

    this.close()
  }

  public setProvider(value: Provider) {
    this.employee.provider = value
    this.employee.slug = ''
  }

  public setSlug(value: string) {
    this.employee.slug = value
  }

  public save = async () => {
    if (!this.employee.isIdentityValid || this.isSaving) return null
    if (this.employee.id) return null

    this.isSaving = true

    try {
      const employee = await employeesApi.create(this.employee.toPayload())
      await this.onCreated?.(employee)

      runInAction(() => {
        this.isOpen = false
        this.reset()
      })

      return employee
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  private reset = () => {
    this.employee = new EmployeeModel()
  }
}
