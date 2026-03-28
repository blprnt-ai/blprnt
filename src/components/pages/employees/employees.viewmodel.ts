import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Employee } from '@/bindings/Employee'
import { employeesApi } from '@/lib/api/employees'
import { AppModel } from '@/models/app.model'

export class EmployeesViewmodel {
  public employees: Employee[] = []
  public isLoading = true
  public errorMessage: string | null = null

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
