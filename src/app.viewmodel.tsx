import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { employeesApi } from './lib/api/employees'
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
      this.runs.disconnect()
      return
    }

    AppModel.instance.setOwner(owner)
    this.runs.connect(owner.id)
    const employees = await employeesApi.list()
    const projects = await projectsApi.list()
    AppModel.instance.setEmployees(employees)
    AppModel.instance.setProjects(projects)
  }

  public get isOnboarded() {
    return AppModel.instance.isOnboarded
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
