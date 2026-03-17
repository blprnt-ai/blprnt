import { makeAutoObservable } from 'mobx'
import type { ProjectDto } from '@/bindings/ProjectDto'
import { projectsApi } from '@/lib/api/projects'
import { ProjectModel } from '@/models/project.model'

export class ProjectFormViewmodel {
  public project: ProjectModel = new ProjectModel()

  constructor() {
    makeAutoObservable(this)
  }

  public init = async (projectId?: string) => {
    if (!projectId) return

    const project = await projectsApi.get(projectId)
    this.setProject(project)
  }

  private setProject = (project: ProjectDto) => {
    this.project = new ProjectModel(project)
  }

  public save = async () => {
    if (!this.project.isDirty) return

    if (!this.project.id) await this.createProject()
    else await this.updateProject()
  }

  private createProject = async () => {
    const project = await projectsApi.create(this.project.toPayload())
    this.setProject(project)
  }

  private updateProject = async () => {
    const project = await projectsApi.update(this.project.id!, this.project.toPayloadPatch())
    this.setProject(project)
  }
}
