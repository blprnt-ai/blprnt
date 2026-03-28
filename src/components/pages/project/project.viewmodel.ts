import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'
import { ProjectModel } from '@/models/project.model'

export class ProjectViewmodel {
  public project: ProjectModel | null = null
  public isEditing = false
  public isLoading = true
  public isSaving = false
  public errorMessage: string | null = null
  private readonly projectId: string
  private originalProject: ProjectDto | null = null

  constructor(projectId: string) {
    this.projectId = projectId

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get canSave() {
    return Boolean(this.project?.isDirty && this.project?.isValid) && !this.isSaving
  }

  public get workingDirectoryCount() {
    return this.project?.workingDirectories.length ?? 0
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const project = await projectsApi.get(this.projectId)

      runInAction(() => {
        this.setProject(project)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load this project.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public startEditing() {
    if (!this.project) return

    this.isEditing = true
  }

  public cancelEditing() {
    if (!this.originalProject) return

    this.project = new ProjectModel(this.originalProject)
    this.isEditing = false
    this.errorMessage = null
  }

  public async save() {
    if (!this.project?.id || !this.project.isDirty) {
      this.isEditing = false
      return this.project
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
    })

    try {
      const project = await projectsApi.update(this.project.id, this.project.toPayloadPatch())

      runInAction(() => {
        this.setProject(project)
        this.isEditing = false
      })

      return this.project
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this project.')
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  private setProject(project: ProjectDto) {
    this.originalProject = project
    this.project = new ProjectModel(project)
    AppModel.instance.upsertProject(project)
  }
}

export const ProjectViewmodelContext = createContext<ProjectViewmodel | null>(null)

export const useProjectViewmodel = () => {
  const viewmodel = useContext(ProjectViewmodelContext)
  if (!viewmodel) throw new Error('ProjectViewmodel not found')

  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}
