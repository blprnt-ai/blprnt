import type { CreateProjectPayload } from '@/bindings/CreateProjectPayload'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProjectPatch } from '@/bindings/ProjectPatch'
import { apiClient } from './fetch'

class ProjectsApi {
  public async list(): Promise<ProjectDto[]> {
    return apiClient.get('/projects')
  }

  public async get(id: string): Promise<ProjectDto> {
    return apiClient.get(`/projects/${id}`)
  }

  public async create(data: CreateProjectPayload): Promise<ProjectDto> {
    return apiClient.post('/projects', {
      body: JSON.stringify(data),
    })
  }

  public async update(id: string, data: ProjectPatch): Promise<ProjectDto> {
    return apiClient.patch(`/projects/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async delete(id: string): Promise<void> {
    return apiClient.delete(`/projects/${id}`)
  }
}

export const projectsApi = new ProjectsApi()
