import { commands } from '@/bindings'

class TauriMemoryApi {
  public async search(projectId: string, query: string, limit: number) {
    const payload = {
      limit: limit as unknown as bigint,
      project_id: projectId,
      query,
    }
    const result = await commands.memorySearch(payload)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async read(projectId: string, path: string) {
    const payload = {
      path,
      project_id: projectId,
    }
    const result = await commands.memoryRead(payload)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async create(projectId: string, content: string) {
    const payload = {
      content,
      project_id: projectId,
    }
    const result = await commands.memoryCreate(payload)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async update(projectId: string, path: string, content: string) {
    const payload = {
      content,
      path,
      project_id: projectId,
    }
    const result = await commands.memoryUpdate(payload)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async delete(projectId: string, path: string) {
    const payload = {
      path,
      project_id: projectId,
    }
    const result = await commands.memoryDelete(payload)
    if (result.status === 'error') throw result.error

    return result.data
  }
}

export const tauriMemoryApi = new TauriMemoryApi()
