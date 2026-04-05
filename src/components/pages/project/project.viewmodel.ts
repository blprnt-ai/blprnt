import { makeAutoObservable, reaction, runInAction, type IReactionDisposer } from 'mobx'
import { createContext, useContext } from 'react'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProjectMemoryListResult, ProjectMemoryReadResult, ProjectMemoryTreeNode } from '@/lib/api/projects'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'
import { ProjectModel } from '@/models/project.model'

export class ProjectViewmodel {
  public activeTab: 'overview' | 'memory' = 'overview'
  public project: ProjectModel | null = null
  public isEditing = false
  public isLoading = true
  public isMemoryFileLoading = false
  public isMemoryLoading = true
  public isSaving = false
  public errorMessage: string | null = null
  public memoryErrorMessage: string | null = null
  public memoryFile: ProjectMemoryReadResult | null = null
  public memoryFileErrorMessage: string | null = null
  public memoryTree: ProjectMemoryListResult | null = null
  public saveState: 'saved' | 'saving' | 'pending' | 'error' = 'saved'
  public selectedMemoryPath: string | null = null
  private readonly projectId: string
  private originalProject: ProjectDto | null = null
  private autosaveTimer: ReturnType<typeof setTimeout> | null = null
  private autosaveDisposer: IReactionDisposer | null = null
  private saveQueued = false
  private readonly autosaveDelayMs = 800

  constructor(projectId: string) {
    this.projectId = projectId

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get canSave() {
    return Boolean(this.project?.isDirty && this.project?.isValid) && !this.isSaving
  }

  public get workingDirectoryCount() {
    return this.project?.workingDirectories.length ?? 0
  }

  public get memoryTreeNodes() {
    return this.memoryTree?.nodes ?? []
  }

  public get hasMemoryFiles() {
    return flattenMemoryFilePaths(this.memoryTreeNodes).length > 0
  }

  public get selectedMemoryFileType() {
    return getMemoryFileType(this.selectedMemoryPath)
  }

  public get canPreviewSelectedMemoryFile() {
    return this.selectedMemoryFileType !== 'unsupported'
  }

  public get selectedMemoryFileName() {
    return this.selectedMemoryPath?.split('/').at(-1) ?? null
  }

  public setActiveTab(value: 'overview' | 'memory') {
    this.activeTab = value
  }

  public startEditing() {
    if (!this.project) return

    this.isEditing = true
  }

  public cancelEditing() {
    if (!this.originalProject) return

    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer)
      this.autosaveTimer = null
    }

    this.saveQueued = false
    this.errorMessage = null
    this.saveState = 'saved'
    this.setProject(this.originalProject)
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    const [projectResult, memoryResult] = await Promise.allSettled([
      projectsApi.get(this.projectId),
      projectsApi.memory(this.projectId),
    ])

    runInAction(() => {
      if (projectResult.status === 'fulfilled') {
        this.setProject(projectResult.value)
      } else {
        this.errorMessage = getErrorMessage(projectResult.reason, 'Unable to load this project.')
      }

      if (memoryResult.status === 'fulfilled') {
        this.applyMemoryTree(memoryResult.value)
      } else {
        this.memoryErrorMessage = getErrorMessage(memoryResult.reason, 'Unable to load project memory.')
      }

      runInAction(() => {
        this.isLoading = false
        this.isMemoryLoading = false
      })
    })

    if (this.selectedMemoryPath) {
      await this.selectMemoryPath(this.selectedMemoryPath)
    }
  }

  public async reloadMemoryTree() {
    runInAction(() => {
      this.isMemoryLoading = true
      this.memoryErrorMessage = null
    })

    try {
      const memoryTree = await projectsApi.memory(this.projectId)

      runInAction(() => {
        this.applyMemoryTree(memoryTree)
      })

      if (this.selectedMemoryPath) {
        await this.selectMemoryPath(this.selectedMemoryPath)
      }
    } catch (error) {
      runInAction(() => {
        this.memoryErrorMessage = getErrorMessage(error, 'Unable to load project memory.')
      })
    } finally {
      runInAction(() => {
        this.isMemoryLoading = false
      })
    }
  }

  public async selectMemoryPath(path: string) {
    runInAction(() => {
      this.selectedMemoryPath = path
      this.isMemoryFileLoading = true
      this.memoryFileErrorMessage = null
    })

    try {
      const file = await projectsApi.readMemoryFile(this.projectId, path)

      runInAction(() => {
        if (this.selectedMemoryPath !== path) return
        this.memoryFile = file
      })
    } catch (error) {
      runInAction(() => {
        if (this.selectedMemoryPath !== path) return
        this.memoryFile = null
        this.memoryFileErrorMessage = getErrorMessage(error, 'Unable to load this memory file.')
      })
    } finally {
      runInAction(() => {
        if (this.selectedMemoryPath !== path) return
        this.isMemoryFileLoading = false
      })
    }
  }

  public async save() {
    if (this.isSaving) {
      this.saveQueued = true
      return this.project
    }

    if (!this.project?.id || !this.project.isDirty) {
      runInAction(() => {
        if (this.saveState !== 'error') this.saveState = 'saved'
      })
      return this.project
    }

    if (!this.project.isValid) {
      runInAction(() => {
        this.errorMessage = 'Project name and at least one working directory are required.'
        this.saveState = 'error'
      })

      return null
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
      this.saveState = 'saving'
    })

    try {
      const project = await projectsApi.update(this.project.id, this.project.toPayloadPatch())

      runInAction(() => {
        this.setProject(project)
        this.saveState = 'saved'
      })

      return this.project
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this project.')
        this.saveState = 'error'
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })

      if (this.saveQueued || (this.project?.isDirty && this.project?.isValid)) {
        this.saveQueued = false
        this.scheduleAutosave(200)
      }
    }
  }

  public destroy() {
    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer)
      this.autosaveTimer = null
    }

    this.autosaveDisposer?.()
    this.autosaveDisposer = null
  }

  private setProject(project: ProjectDto) {
    this.originalProject = project
    this.project = new ProjectModel(project)
    this.isEditing = false
    this.setupAutosave()
    AppModel.instance.upsertProject(project)
  }

  private applyMemoryTree(memoryTree: ProjectMemoryListResult) {
    const availablePaths = flattenMemoryFilePaths(memoryTree.nodes)
    const selectedPath = selectDefaultMemoryPath(availablePaths)

    this.memoryTree = memoryTree
    this.memoryErrorMessage = null

    if (availablePaths.length === 0) {
      this.selectedMemoryPath = null
      this.memoryFile = null
      this.memoryFileErrorMessage = null
      return
    }

    if (this.selectedMemoryPath && availablePaths.includes(this.selectedMemoryPath)) return

    this.selectedMemoryPath = selectedPath
    this.memoryFile = null
    this.memoryFileErrorMessage = null
  }

  private setupAutosave() {
    this.autosaveDisposer?.()
    this.autosaveDisposer = reaction(
      () => (this.project?.isDirty ? JSON.stringify(this.project.toPayloadPatch()) : ''),
      (payload) => {
        if (!payload) return

        if (!this.project?.isValid) {
          runInAction(() => {
            this.errorMessage = 'Project name and at least one working directory are required.'
            this.saveState = 'error'
          })
          return
        }

        this.scheduleAutosave()
      },
    )
  }

  private scheduleAutosave(delay = this.autosaveDelayMs) {
    if (this.autosaveTimer) clearTimeout(this.autosaveTimer)

    runInAction(() => {
      if (!this.isSaving) this.saveState = 'pending'
    })

    this.autosaveTimer = setTimeout(() => {
      this.autosaveTimer = null
      void this.save()
    }, delay)
  }
}

export const ProjectViewmodelContext = createContext<ProjectViewmodel | null>(null)

export const useProjectViewmodel = () => {
  const viewmodel = useContext(ProjectViewmodelContext)
  if (!viewmodel) throw new Error('ProjectViewmodel not found')

  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}

const flattenMemoryFilePaths = (nodes: ProjectMemoryTreeNode[]): string[] => {
  return nodes.flatMap((node) => {
    if (node.type === 'file') return [node.path]

    return flattenMemoryFilePaths(node.children)
  })
}

const selectDefaultMemoryPath = (paths: string[]) => {
  return paths.find((path) => path === 'SUMMARY.md') ?? paths[0] ?? null
}

const getMemoryFileType = (path: string | null) => {
  const normalizedPath = path?.toLowerCase() ?? ''

  if (normalizedPath.endsWith('.md') || normalizedPath.endsWith('.markdown')) return 'markdown'
  if (normalizedPath.endsWith('.txt') || normalizedPath.endsWith('.log') || normalizedPath.endsWith('.json') || normalizedPath.endsWith('.yaml') || normalizedPath.endsWith('.yml')) {
    return 'text'
  }

  return 'unsupported'
}
