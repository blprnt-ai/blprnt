import { commands } from '@/bindings'

class TauriPersonalitiesApi {
  public async list() {
    const result = await commands.personalityList()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async create(name: string, description: string, systemPrompt: string) {
    const result = await commands.personalityCreate(name, description, systemPrompt)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async update(id: string, name: string, description: string, systemPrompt: string) {
    const result = await commands.personalityUpdate(id, name, description, systemPrompt)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async delete(id: string) {
    const result = await commands.personalityDelete(id)
    if (result.status === 'error') throw result.error
  }
}

export const tauriPersonalitiesApi = new TauriPersonalitiesApi()
