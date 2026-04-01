import type { Skill } from '@/bindings/Skill'
import { apiClient } from './fetch'

class SkillsApi {
  public async list(): Promise<Skill[]> {
    return apiClient.get('/skills')
  }
}

export const skillsApi = new SkillsApi()
