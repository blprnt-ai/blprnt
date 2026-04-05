import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'

export class ProjectsViewmodel {
  public projects: ProjectDto[] = []
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
      const projects = await projectsApi.list()

      runInAction(() => {
        this.projects = projects
        AppModel.instance.setProjects(projects)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load projects.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }
}

export const ProjectsViewmodelContext = createContext<ProjectsViewmodel | null>(null)

export const useProjectsViewmodel = () => {
  const viewmodel = useContext(ProjectsViewmodelContext)
  if (!viewmodel) throw new Error('ProjectsViewmodel not found')

  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}
