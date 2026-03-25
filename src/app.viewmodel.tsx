import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import { employeesApi } from './lib/api/employees'
import { issuesApi } from './lib/api/issues'
import { projectsApi } from './lib/api/projects'
import { providersApi } from './lib/api/providers'
import { AppModel } from './models/app.model'

export class AppViewmodel {
  public isLoading = true

  constructor() {
    makeAutoObservable(this)
  }

  public async init() {
    try {
      const owner = await employeesApi.getOwner()
      if (!owner) return

      AppModel.instance.setOwner(owner)

      const providers = await providersApi.list()
      AppModel.instance.setHasProvider(providers.length > 0)

      const projects = await projectsApi.list()
      AppModel.instance.setHasProjects(projects.length > 0)

      const issues = await issuesApi.list()
      AppModel.instance.setHasIssues(issues.length > 0)
    } catch (error) {
      console.error(error)
      toast.error('Failed to initialize app. Please try again.')
    } finally {
      this.isLoading = false
    }
  }

  public get isOnboarded() {
    return AppModel.instance.isOnboarded
  }
}

export const AppViewmodelContext = createContext<AppViewmodel | null>(null)
export const useAppViewmodel = () => {
  const appViewmodel = useContext(AppViewmodelContext)
  if (!appViewmodel) throw new Error('AppViewmodel not found')

  return appViewmodel
}
