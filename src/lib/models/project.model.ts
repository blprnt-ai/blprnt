import { flow, makeAutoObservable, observable } from 'mobx'
import type { ProjectPatchV2, ProjectRecord } from '@/bindings'
import { tauriProjectApi } from '@/lib/api/tauri/project.api'
import { EventType, globalEventBus } from '@/lib/events'
import type { InternalEvent } from '@/lib/events/event-bus'
import { newProjectId } from '@/lib/utils/default-models'

export interface CreateProjectArgs {
  name: string
  workingDirectories: string[]
  agentPrimer: string
}

export class ProjectModel {
  private static registry = new Map<string, ProjectModel>()
  private static subscribed = false

  public id: string = newProjectId
  public name: string = ''
  public workingDirectories: string[] = observable.array([])
  public agentPrimer: string | null | undefined = ''
  public createdAt: number = Date.now()
  public updatedAt: number = Date.now()

  constructor(model: ProjectRecord) {
    this.id = model.id
    this.name = model.name
    this.workingDirectories = model.working_directories
    this.agentPrimer = model.agent_primer
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at

    makeAutoObservable(this, {}, { autoBind: true })
  }

  updateFrom = (model: ProjectRecord) => {
    this.name = model.name
    this.workingDirectories = model.working_directories
    this.agentPrimer = model.agent_primer
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
  }

  private static ensureSubscribed = () => {
    if (ProjectModel.subscribed) return
    ProjectModel.subscribed = true

    globalEventBus.subscribe(EventType.Internal, (event) => {
      const internal = event.payload.event as InternalEvent
      if (
        internal.type === 'project_added' ||
        internal.type === 'project_updated' ||
        internal.type === 'project_removed'
      ) {
        void ProjectModel.handleInternalEvent(internal)
      }
    })
  }

  private static handleInternalEvent = async (event: InternalEvent) => {
    if (!('projectId' in event)) return

    if (event.type === 'project_removed') {
      ProjectModel.registry.delete(event.projectId)
      return
    }

    if (event.type === 'project_added' || event.type === 'project_updated') {
      const result = await tauriProjectApi.get(event.projectId)
      ProjectModel.getOrCreate(result)
    }
  }

  static getOrCreate = (model: ProjectRecord) => {
    ProjectModel.ensureSubscribed()
    const existing = ProjectModel.registry.get(model.id)
    if (existing) {
      existing.updateFrom(model)
      return existing
    }

    const instance = new ProjectModel(model)
    ProjectModel.registry.set(model.id, instance)
    return instance
  }

  static list = async () => {
    const result = await tauriProjectApi.list()

    return result.map((project) => ProjectModel.getOrCreate(project))
  }

  static get = async (projectId: string) => {
    const result = await tauriProjectApi.get(projectId)

    return ProjectModel.getOrCreate(result)
  }

  static create = async ({ name, workingDirectories, agentPrimer }: CreateProjectArgs) => {
    const result = await tauriProjectApi.create(name, workingDirectories, agentPrimer)

    return ProjectModel.getOrCreate(result)
  }

  update = async (patch: ProjectPatchV2) => {
    const result = await tauriProjectApi.update(this.id, patch)
    this.updateFrom(result)

    return this
  }

  delete = flow(function* (this: ProjectModel) {
    try {
      yield tauriProjectApi.delete(this.id)

      return true
    } catch {
      return false
    }
  })
}
