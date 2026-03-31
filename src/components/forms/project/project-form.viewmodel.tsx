import { makeAutoObservable, runInAction } from 'mobx'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { projectsApi } from '@/lib/api/projects'
import { ProjectModel } from '@/models/project.model'

export class ProjectFormViewmodel {
  public isOpen = false
  public isSaving = false
  public project: ProjectModel = new ProjectModel()
  private onCreated?: (project: ProjectDto) => Promise<void> | void

  constructor(onCreated?: (project: ProjectDto) => Promise<void> | void) {
    this.onCreated = onCreated
    makeAutoObservable(this)
    this.project.addWorkingDirectory()
  }

  public get canSave() {
    return this.project.isValid && !this.isSaving
  }

  public open = () => {
    this.reset()
    this.isOpen = true
  }

  public close = () => {
    if (this.isSaving) return
    this.isOpen = false
    this.reset()
  }

  public setOpen = (isOpen: boolean) => {
    if (isOpen) {
      this.open()
      return
    }

    this.close()
  }

  public save = async () => {
    if (!this.project.isValid || this.isSaving) return null
    if (this.project.id) return null

    this.isSaving = true

    try {
      const project = await projectsApi.create(this.project.toPayload())
      await this.onCreated?.(project)

      runInAction(() => {
        this.isOpen = false
        this.reset()
      })

      return project
    } finally {
      runInAction(() => {
        this.isSaving = false
      })
    }
  }

  private reset = () => {
    this.project = new ProjectModel()
    this.project.addWorkingDirectory()
  }
}
