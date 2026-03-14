import { flow, makeAutoObservable } from 'mobx'
import type {
  PlanCreateArgs,
  PlanDocumentStatus,
  PlanGetPayload as PlanDto,
  PlanListQuery,
  PlanTodoItem,
  ProjectPlanListItem,
} from '@/bindings'
import { tauriProjectApi } from '@/lib/api/tauri/project.api'
import { tauriSessionApi } from '@/lib/api/tauri/session.api'
import { EventType, globalEventBus } from '@/lib/events'
import type { InternalEvent } from '@/lib/events/event-bus'

export class PlanModel {
  private static registry = new Map<string, PlanModel>()
  private static subscribed = false

  public id: string
  public projectId: string
  public name: string
  public description: string
  public content: string
  public createdAt: string
  public updatedAt: string
  public todos: PlanTodoItem[]
  public status: PlanDocumentStatus = 'pending'

  constructor(projectId: string, model: PlanDto) {
    this.projectId = projectId
    this.id = model.id
    this.name = model.name
    this.description = model.description
    this.content = model.content
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
    this.todos = model.todos ?? []
    this.status = model.status ?? 'pending'

    makeAutoObservable(this, { projectId: false }, { autoBind: true })
  }

  updateFrom = (model: PlanDto) => {
    this.id = model.id
    this.name = model.name
    this.description = model.description
    this.content = model.content
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
    this.todos = model.todos ?? []
    this.status = model.status ?? 'pending'
  }

  setStatus = (status: PlanDocumentStatus) => {
    this.status = status
  }

  get inProgress() {
    return this.status === 'in_progress'
  }

  get pending() {
    return this.status === 'pending'
  }

  get complete() {
    return this.status === 'completed'
  }

  get isCompletable() {
    return this.todos.every((todo) => todo.status === 'complete')
  }

  private static registryKey = (projectId: string, planId: string) => `${projectId}:${planId}`

  private static ensureSubscribed = () => {
    if (PlanModel.subscribed) return
    PlanModel.subscribed = true

    globalEventBus.subscribe(EventType.Internal, (event) => {
      const internal = event.payload.event as InternalEvent
      if (internal.type === 'plan_updated') {
        void PlanModel.handleInternalEvent(internal)
      }
    })
  }

  private static handleInternalEvent = async (event: InternalEvent) => {
    if (event.type !== 'plan_updated') return
    const projectId = event.projectId
    if (projectId) {
      try {
        const result = await tauriProjectApi.planGet(projectId, event.planId)
        PlanModel.getOrCreate(projectId, result)
      } catch (error) {
        console.error(error)
      }
    } else {
      const matching = Array.from(PlanModel.registry.values()).filter((plan) => plan.id === event.planId)
      await Promise.all(matching.map((plan) => plan.refresh()))
    }
  }

  static getOrCreate = (projectId: string, model: PlanDto) => {
    PlanModel.ensureSubscribed()
    const key = PlanModel.registryKey(projectId, model.id)
    const existing = PlanModel.registry.get(key)
    if (existing) {
      existing.updateFrom(model)
      return existing
    }

    const instance = new PlanModel(projectId, model)
    PlanModel.registry.set(key, instance)
    return instance
  }

  static get = async (projectId: string, planId: string) => {
    try {
      const result = await tauriProjectApi.planGet(projectId, planId)
      return PlanModel.getOrCreate(projectId, result)
    } catch (error) {
      console.error(error)
      return null
    }
  }

  static listForProject = async (
    projectId: string,
    query: PlanListQuery | null = null,
  ): Promise<ProjectPlanListItem[]> => {
    const result = await tauriProjectApi.planList(projectId, query)
    return result.items
  }

  static createForProject = async (projectId: string, args: PlanCreateArgs) => {
    const result = await tauriProjectApi.planCreate(projectId, args)
    return PlanModel.getOrCreate(projectId, result)
  }

  static cancelForSession = async (sessionId: string, planId: string) => {
    await tauriSessionApi.cancelPlan(sessionId, planId)
  }

  static deleteForSession = async (sessionId: string, planId: string) => {
    await tauriSessionApi.deletePlan(sessionId, planId)
  }

  static archiveForProject = async (projectId: string, planId: string) => {
    await tauriProjectApi.planCancel(projectId, planId)
  }

  static deleteForProject = async (projectId: string, planId: string) => {
    await tauriProjectApi.planDelete(projectId, planId)
  }

  static assignToSession = async (sessionId: string, planId: string) => {
    await tauriSessionApi.assignPlanToSession(sessionId, planId)
  }

  static unassignFromSession = async (sessionId: string, planId: string) => {
    await tauriSessionApi.unassignPlanFromSession(sessionId, planId)
  }

  refresh = flow(function* (this: PlanModel) {
    const result = yield tauriProjectApi.planGet(this.projectId, this.id)
    this.updateFrom(result)
    return this
  })

  update = flow(function* (
    this: PlanModel,
    patch: {
      name?: string | null
      description?: string | null
      content?: string | null
      todos?: PlanTodoItem[] | null
    },
  ) {
    const result = yield tauriProjectApi.planUpdate(this.projectId, this.id, patch)
    this.name = result.name
    this.description = result.description
    this.updatedAt = result.updated_at

    const canonical = yield tauriProjectApi.planGet(this.projectId, this.id)
    this.updateFrom(canonical)

    return this
  })
}
