import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { Provider } from '@/bindings/Provider'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'

export class EmployeeViewmodel {
  public employee: EmployeeModel | null = null
  public isEditing = false
  public isLoading = true
  public isSaving = false
  public errorMessage: string | null = null
  private readonly employeeId: string
  private originalEmployee: Employee | null = null

  constructor(employeeId: string) {
    this.employeeId = employeeId

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get canSave() {
    return Boolean(this.employee?.isDirty) && !this.isSaving
  }

  public get capabilitiesValue() {
    return this.employee?.capabilities.join(', ') ?? ''
  }

  public get showsAgentConfiguration() {
    return this.employee?.kind === 'agent'
  }

  public get roleValue() {
    if (!this.employee) return ''
    if (typeof this.employee.role === 'string') return this.employee.role
    if ('custom' in this.employee.role) return this.employee.role.custom

    return ''
  }

  public get reportsTo() {
    return this.originalEmployee?.reports_to ?? null
  }

  public get chainOfCommand() {
    return this.originalEmployee?.chain_of_command ?? []
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const employee = await employeesApi.get(this.employeeId)

      runInAction(() => {
        this.setEmployee(employee)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load this employee.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public startEditing() {
    if (!this.employee) return

    this.isEditing = true
  }

  public cancelEditing() {
    if (!this.originalEmployee) return

    this.employee = new EmployeeModel(this.originalEmployee)
    this.isEditing = false
    this.errorMessage = null
  }

  public async save() {
    if (!this.employee?.id || !this.employee.isDirty) {
      this.isEditing = false
      return this.employee
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
    })

    try {
      const employee = await employeesApi.update(this.employee.id, this.employee.toPayloadPatch())

      runInAction(() => {
        this.setEmployee(employee)
        this.isEditing = false
      })

      return this.employee
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this employee.')
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  public setCapabilities(value: string) {
    if (!this.employee) return

    this.employee.capabilities = value
      .split(',')
      .map((part) => part.trim())
      .filter(Boolean)
  }

  public setRole(value: string) {
    if (!this.employee) return

    this.employee.role = parseRole(value)
  }

  public setProvider(value: Provider) {
    if (!this.employee) return

    this.employee.provider = value
  }

  private setEmployee(employee: Employee) {
    this.originalEmployee = employee
    this.employee = new EmployeeModel(employee)

    if (AppModel.instance.owner?.id === employee.id) {
      AppModel.instance.setOwner(employee)
      return
    }

    AppModel.instance.upsertEmployee(employee)
  }
}

export const EmployeeViewmodelContext = createContext<EmployeeViewmodel | null>(null)

export const useEmployeeViewmodel = () => {
  const viewmodel = useContext(EmployeeViewmodelContext)
  if (!viewmodel) throw new Error('EmployeeViewmodel not found')

  return viewmodel
}

const parseRole = (value: string): EmployeeRole => {
  if (value === 'owner' || value === 'ceo' || value === 'manager' || value === 'staff') return value

  return { custom: value }
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}
