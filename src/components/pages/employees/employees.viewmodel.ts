import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { Employee } from '@/bindings/Employee'
import type { OrgChart } from '@/bindings/OrgChart'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'
import {
  DEFAULT_EMPLOYEE_LIBRARY_BASE_URL,
  type EmployeeLibraryItem,
  type EmployeeLibraryManifest,
  loadEmployeeLibraryManifest,
  resolveEmployeeLibraryImportBaseUrl,
} from './employee-library'

export type EmployeesView = 'list' | 'org-chart'

export class EmployeesViewmodel {
  public employees: Employee[] = []
  public orgChart: OrgChart[] = []
  public isLoading = true
  public isImporting = false
  public isLoadingImportManifest = false
  public errorMessage: string | null = null
  public importManifestError: string | null = null
  public importBaseUrl = DEFAULT_EMPLOYEE_LIBRARY_BASE_URL
  public importSlug = ''
  public importManifest: EmployeeLibraryManifest | null = null
  public importForce = false
  public importSkipDuplicateSkills = true
  public importForceSkills = false
  public activeView: EmployeesView = 'list'

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
      const [employees, orgChart] = await Promise.all([
        employeesApi.list(),
        employeesApi.orgChart(),
        this.loadImportManifest(),
      ])

      runInAction(() => {
        this.employees = employees
        this.orgChart = orgChart
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

  public setImportBaseUrl(baseUrl: string) {
    this.importBaseUrl = baseUrl
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

  public setActiveView(view: EmployeesView) {
    this.activeView = view
  }

  public get canImport() {
    return this.selectedImportEmployee !== null && !this.isImporting && !this.isLoadingImportManifest
  }

  public get importEmployeeOptions() {
    return this.importManifest?.employees ?? []
  }

  public get selectedImportEmployee(): EmployeeLibraryItem | null {
    if (!this.importSlug) return null

    return this.importEmployeeOptions.find((employee) => employee.id === this.importSlug) ?? null
  }

  public async loadImportManifest() {
    const baseUrl = this.importBaseUrl.trim()

    runInAction(() => {
      this.isLoadingImportManifest = true
      this.importManifestError = null
    })

    try {
      const manifest = await loadEmployeeLibraryManifest(baseUrl)

      runInAction(() => {
        this.importManifest = manifest
        if (!manifest.employees.some((employee) => employee.id === this.importSlug)) {
          this.importSlug = ''
        }
      })
    } catch (error) {
      const message = getErrorMessage(error, 'Unable to load employee manifest.')

      runInAction(() => {
        this.importManifest = null
        this.importSlug = ''
        this.importManifestError = message
      })
    } finally {
      runInAction(() => {
        this.isLoadingImportManifest = false
      })
    }
  }

  public async importEmployee() {
    const slug = this.importSlug.trim()
    let baseUrl = this.importBaseUrl.trim()
    if (!slug || !baseUrl || this.isImporting) return

    runInAction(() => {
      this.isImporting = true
      this.errorMessage = null
    })

    try {
      baseUrl = resolveEmployeeLibraryImportBaseUrl(baseUrl)

      const employee = await employeesApi.import({
        base_url: baseUrl,
        force: this.importForce,
        force_skills: this.importForceSkills,
        skip_duplicate_skills: this.importSkipDuplicateSkills,
        slug,
      })
      const orgChart = await employeesApi.orgChart()

      runInAction(() => {
        this.importSlug = ''
        this.importForce = false
        this.importSkipDuplicateSkills = true
        this.importForceSkills = false
        this.employees = sortEmployees([
          ...this.employees.filter((candidate) => candidate.id !== employee.id),
          employee,
        ])
        this.orgChart = orgChart
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
