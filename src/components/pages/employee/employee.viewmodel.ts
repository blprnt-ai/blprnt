import { type IReactionDisposer, makeAutoObservable, reaction, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { Provider } from '@/bindings/Provider'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'

export class EmployeeViewmodel {
  public employee: EmployeeModel | null = null
  public isLoading = true
  public isSaving = false
  public errorMessage: string | null = null
  public saveState: 'saved' | 'saving' | 'pending' | 'error' = 'saved'
  public lastSavedAt: Date | null = null
  private readonly employeeId: string
  private originalEmployee: Employee | null = null
  private autosaveTimer: ReturnType<typeof setTimeout> | null = null
  private autosaveDisposer: IReactionDisposer | null = null
  private saveQueued = false
  private readonly autosaveDelayMs = 800

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

  public get saveStatusLabel() {
    switch (this.saveState) {
      case 'saving':
        return 'Saving changes...'
      case 'pending':
        return 'Changes pending'
      case 'error':
        return 'Autosave failed'
      default:
        if (!this.lastSavedAt) return 'Ready'

        return `Saved ${formatTime(this.lastSavedAt)}`
    }
  }

  public get saveStatusHint() {
    switch (this.saveState) {
      case 'saving':
        return 'Updates are being written now.'
      case 'pending':
        return 'Keep editing. Everything saves automatically.'
      case 'error':
        return this.errorMessage ?? 'We could not save your latest changes.'
      default:
        return 'This page saves in place as you edit.'
    }
  }

  public get capabilitiesValue() {
    return this.employee?.capabilities.join(', ') ?? ''
  }

  public get showsAgentConfiguration() {
    return this.employee?.kind === 'agent'
  }

  public get isHumanEmployee() {
    return this.employee?.kind === 'person'
  }

  public get isOwnerEmployee() {
    return this.employee?.role === 'owner'
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

  public async save() {
    if (this.isSaving) {
      this.saveQueued = true
      return this.employee
    }

    if (!this.employee?.id || !this.employee.isDirty) {
      runInAction(() => {
        if (this.saveState !== 'error') this.saveState = 'saved'
      })
      return this.employee
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
      this.saveState = 'saving'
    })

    try {
      const employee = await employeesApi.update(this.employee.id, this.employee.toPayloadPatch())

      runInAction(() => {
        this.setEmployee(employee)
        this.lastSavedAt = new Date()
        this.saveState = 'saved'
      })

      return this.employee
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this employee.')
        this.saveState = 'error'
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })

      if (this.saveQueued || this.employee?.isDirty) {
        this.saveQueued = false
        this.scheduleAutosave(200)
      }
    }
  }

  public destroy() {
    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer)
      this.autosaveTimer = null
    }

    this.autosaveDisposer?.()
    this.autosaveDisposer = null
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

  public setSlug(value: string) {
    if (!this.employee) return

    this.employee.slug = value
  }

  private setEmployee(employee: Employee) {
    this.originalEmployee = employee
    this.employee = new EmployeeModel(employee)
    this.setupAutosave()

    if (AppModel.instance.owner?.id === employee.id) {
      AppModel.instance.setOwner(employee)
      return
    }

    AppModel.instance.upsertEmployee(employee)
  }

  private setupAutosave() {
    this.autosaveDisposer?.()
    this.autosaveDisposer = reaction(
      () => (this.employee?.isDirty ? JSON.stringify(this.employee.toPayloadPatch()) : ''),
      (payload) => {
        if (!payload) return

        this.scheduleAutosave()
      },
    )
  }

  private scheduleAutosave(delay = this.autosaveDelayMs) {
    if (this.autosaveTimer) clearTimeout(this.autosaveTimer)

    runInAction(() => {
      if (!this.isSaving) this.saveState = 'pending'
    })

    this.autosaveTimer = setTimeout(() => {
      this.autosaveTimer = null
      void this.save()
    }, delay)
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

const formatTime = (value: Date) =>
  value.toLocaleTimeString([], {
    hour: 'numeric',
    minute: '2-digit',
  })
