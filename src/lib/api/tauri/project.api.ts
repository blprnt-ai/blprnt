import {
  commands,
  type PlanCreateArgs,
  type PlanDocumentStatus,
  type PlanListQuery,
  type PlanTodoItem,
  type ProjectPatchV2,
} from '@/bindings'
import { EventType, globalEventBus } from '@/lib/events'

class TauriProjectApi {
  public async create(name: string, workingDirectories: string[], agentPrimer: string) {
    const result = await commands.newProject(name, workingDirectories, agentPrimer)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        projectId: result.data.id,
        type: 'project_added',
      },
    })

    return result.data
  }

  public async update(projectId: string, project: ProjectPatchV2) {
    const result = await commands.editProject(projectId, project)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        projectId: projectId,
        type: 'project_updated',
      },
    })

    return result.data
  }

  public async get(projectId: string) {
    const result = await commands.getProject(projectId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async list() {
    const result = await commands.listProjects()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async delete(projectId: string) {
    const result = await commands.deleteProject(projectId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        projectId: projectId,
        type: 'project_removed',
      },
    })
  }

  public async planGet(projectId: string, planId: string) {
    const result = await commands.planGet(projectId, planId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async planList(projectId: string, query: PlanListQuery | null = null) {
    const result = await commands.planList(projectId, query)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async planCreate(projectId: string, args: PlanCreateArgs) {
    const result = await commands.planCreate(projectId, args)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        planId: result.data.id,
        projectId,
        type: 'plan_updated',
      },
    })

    return result.data
  }

  public async planUpdate(
    projectId: string,
    planId: string,
    patch: {
      name?: string | null
      description?: string | null
      content?: string | null
      todos?: PlanTodoItem[] | null
      status?: PlanDocumentStatus | null
    },
  ) {
    const result = await commands.planUpdate(projectId, {
      content: patch.content ?? null,
      content_patch: null,
      description: patch.description ?? null,
      id: planId,
      name: patch.name ?? null,
      status: patch.status ?? null,
      todos: patch.todos ?? null,
    })
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        planId: planId,
        projectId: projectId,
        type: 'plan_updated',
      },
    })

    return result.data
  }

  public async planCancel(projectId: string, planId: string) {
    const result = await commands.planCancel(projectId, planId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        planId,
        projectId,
        type: 'plan_updated',
      },
    })
  }

  public async planDelete(projectId: string, planId: string) {
    const result = await commands.planDelete(projectId, planId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        planId,
        projectId,
        type: 'plan_updated',
      },
    })
  }
}

export const tauriProjectApi = new TauriProjectApi()
