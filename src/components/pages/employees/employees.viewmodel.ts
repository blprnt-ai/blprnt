import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { Employee } from '@/bindings/Employee'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'

export class EmployeesViewmodel {
  public employees: Employee[] = []
  public isLoading = true
  public isImporting = false
  public errorMessage: string | null = null
  public importSlug = ''
  public importForce = false
  public importSkipDuplicateSkills = true
  public importForceSkills = false

  constructor() {
    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const employees = await employeesApi.list()

      runInAction(() => {
        this.employees = employees
        AppModel.instance.setEmployees(employees)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load employees.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public setImportSlug(slug: string) {
    this.importSlug = slug
  }

  public setImportForce(force: boolean) {
    this.importForce = force
  }

  public setImportSkipDuplicateSkills(skipDuplicateSkills: boolean) {
    this.importSkipDuplicateSkills = skipDuplicateSkills
  }

  public setImportForceSkills(forceSkills: boolean) {
    this.importForceSkills = forceSkills
  }

  public get canImport() {
    return this.importSlug.trim().length > 0 && !this.isImporting
  }

  public async importEmployee() {
    const slug = this.importSlug.trim()
    if (!slug || this.isImporting) return

    runInAction(() => {
      this.isImporting = true
      this.errorMessage = null
    })

    try {
      const employee = await employeesApi.import({
        force: this.importForce,
        force_skills: this.importForceSkills,
        skip_duplicate_skills: this.importSkipDuplicateSkills,
        slug,
      })

      runInAction(() => {
        this.importSlug = ''
        this.importForce = false
        this.importSkipDuplicateSkills = true
        this.importForceSkills = false
        this.employees = sortEmployees([...this.employees.filter((candidate) => candidate.id !== employee.id), employee])
        AppModel.instance.upsertEmployee(employee)
      })

      toast.success(`Imported ${employee.name}.`)
    } catch (error) {
      const message = getErrorMessage(error, 'Unable to import employee.')

      runInAction(() => {
        this.errorMessage = message
      })

      toast.error(message)
    } finally {
      runInAction(() => {
        this.isImporting = false
      })
    }
  }
}

export const EmployeesViewmodelContext = createContext<EmployeesViewmodel | null>(null)

export const useEmployeesViewmodel = () => {
  const viewmodel = useContext(EmployeesViewmodelContext)
  if (!viewmodel) throw new Error('EmployeesViewmodel not found')

  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}

const sortEmployees = (employees: Employee[]) => {
  return [...employees].sort((left, right) => {
    if (left.role === 'owner' && right.role !== 'owner') return -1
    if (left.role !== 'owner' && right.role === 'owner') return 1

    return left.name.localeCompare(right.name)
  })
}
