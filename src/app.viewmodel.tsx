import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { employeesApi } from './lib/api/employees'
import { issuesApi } from './lib/api/issues'
import { projectsApi } from './lib/api/projects'
import { AppModel } from './models/app.model'
import { RunsViewmodel } from './runs.viewmodel'

export class AppViewmodel {
  public runs = new RunsViewmodel()

  constructor() {
    makeAutoObservable(this)
  }

  public init = async () => {
    const owner = await employeesApi.getOwner()
    if (!owner) {
      AppModel.instance.setEmployees([])
      AppModel.instance.setProjects([])
      AppModel.instance.setIsOnboarded(false)
      this.runs.disconnect()
      return
    }

    AppModel.instance.setOwner(owner)
    this.runs.connect(owner.id)
    const employees = await employeesApi.list()
    const projects = await projectsApi.list()
    const issues = await issuesApi.list()
    AppModel.instance.setEmployees(employees)
    AppModel.instance.setProjects(projects)
    AppModel.instance.setIsOnboarded(issues.length > 0)
  }

  public get isOnboarded() {
    return AppModel.instance.isOnboarded
  }

  public get hasOwner() {
    return AppModel.instance.hasOwner
  }

  public setIsOnboarded(isOnboarded: boolean) {
    AppModel.instance.setIsOnboarded(isOnboarded)
  }
}

export const AppViewmodelContext = createContext<AppViewmodel | null>(null)
export const useAppViewmodel = () => {
  const appViewmodel = useContext(AppViewmodelContext)
  if (!appViewmodel) throw new Error('AppViewmodel not found')

  return appViewmodel
}
