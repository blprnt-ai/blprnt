import type { CreateProjectPayload } from '@/bindings/CreateProjectPayload'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProjectPatch } from '@/bindings/ProjectPatch'
import { apiClient } from './fetch'

export interface ProjectMemoryListResult {
  root_path: string
  nodes: ProjectMemoryTreeNode[]
}

export type ProjectMemoryTreeNode =
  | {
      type: 'directory'
      name: string
      path: string
      children: ProjectMemoryTreeNode[]
    }
  | {
      type: 'file'
      name: string
      path: string
    }

export interface ProjectMemoryReadResult {
  path: string
  content: string
}

export interface ProjectMemorySearchResult {
  title: string
  content: string
  score: number
  path?: string | null
}

export interface ProjectPlanListItem {
  path: string
  title: string
  filename: string
  updated_at: string
  is_superseded: boolean
}

export interface ProjectPlansListResult {
  plans: ProjectPlanListItem[]
}

export interface ProjectPlanReadResult {
  path: string
  mime_type: string
  is_previewable: boolean
  content?: string | null
}

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

  public async memory(id: string): Promise<ProjectMemoryListResult> {
    return apiClient.get(`/projects/${id}/memory`)
  }

  public async readMemoryFile(id: string, path: string): Promise<ProjectMemoryReadResult> {
    return apiClient.get(`/projects/${id}/memory/file?path=${encodeURIComponent(path)}`)
  }

  public async searchMemory(id: string, query: string, limit = 10): Promise<ProjectMemorySearchResult[]> {
    return apiClient.post(`/projects/${id}/memory/search`, {
      body: JSON.stringify({
        limit,
        query,
      }),
    })
  }

  public async plans(id: string): Promise<ProjectPlansListResult> {
    return apiClient.get(`/projects/${id}/plans`)
  }

  public async readPlanFile(id: string, path: string): Promise<ProjectPlanReadResult> {
    return apiClient.get(`/projects/${id}/plans/file?path=${encodeURIComponent(path)}`)
  }

  public async delete(id: string): Promise<void> {
    return apiClient.delete(`/projects/${id}`)
  }

  public async nukeDatabase(): Promise<void> {
    return apiClient.delete('/dev/database')
  }
}

export const projectsApi = new ProjectsApi()
