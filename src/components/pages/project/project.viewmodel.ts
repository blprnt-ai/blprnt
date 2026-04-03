import { makeAutoObservable, reaction, runInAction, type IReactionDisposer } from 'mobx'
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
  public saveState: 'saved' | 'saving' | 'pending' | 'error' = 'saved'
  private readonly projectId: string
  private originalProject: ProjectDto | null = null
  private autosaveTimer: ReturnType<typeof setTimeout> | null = null
  private autosaveDisposer: IReactionDisposer | null = null
  private saveQueued = false
  private readonly autosaveDelayMs = 800

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

  public startEditing() {
    if (!this.project) return

    this.isEditing = true
  }

  public cancelEditing() {
    if (!this.originalProject) return

    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer)
      this.autosaveTimer = null
    }

    this.saveQueued = false
    this.errorMessage = null
    this.saveState = 'saved'
    this.setProject(this.originalProject)
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

  public async save() {
    if (this.isSaving) {
      this.saveQueued = true
      return this.project
    }

    if (!this.project?.id || !this.project.isDirty) {
      runInAction(() => {
        if (this.saveState !== 'error') this.saveState = 'saved'
      })
      return this.project
    }

    if (!this.project.isValid) {
      runInAction(() => {
        this.errorMessage = 'Project name and at least one working directory are required.'
        this.saveState = 'error'
      })

      return null
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
      this.saveState = 'saving'
    })

    try {
      const project = await projectsApi.update(this.project.id, this.project.toPayloadPatch())

      runInAction(() => {
        this.setProject(project)
        this.saveState = 'saved'
      })

      return this.project
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this project.')
        this.saveState = 'error'
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })

      if (this.saveQueued || (this.project?.isDirty && this.project?.isValid)) {
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

  private setProject(project: ProjectDto) {
    this.originalProject = project
    this.project = new ProjectModel(project)
    this.isEditing = false
    this.setupAutosave()
    AppModel.instance.upsertProject(project)
  }

  private setupAutosave() {
    this.autosaveDisposer?.()
    this.autosaveDisposer = reaction(
      () => (this.project?.isDirty ? JSON.stringify(this.project.toPayloadPatch()) : ''),
      (payload) => {
        if (!payload) return

        if (!this.project?.isValid) {
          runInAction(() => {
            this.errorMessage = 'Project name and at least one working directory are required.'
            this.saveState = 'error'
          })
          return
        }

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
