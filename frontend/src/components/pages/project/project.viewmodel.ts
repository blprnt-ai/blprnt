import { type IReactionDisposer, makeAutoObservable, reaction, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type {
  ProjectMemoryListResult,
  ProjectMemoryReadResult,
  ProjectMemorySearchResult,
  ProjectMemoryTreeNode,
  ProjectPlanListItem,
  ProjectPlanReadResult,
  ProjectPlansListResult,
} from '@/lib/api/projects'
import { projectsApi } from '@/lib/api/projects'
import { AppModel } from '@/models/app.model'
import { ProjectModel } from '@/models/project.model'

export class ProjectViewmodel {
  public activeTab: 'overview' | 'memory' | 'plans' = 'overview'
  public project: ProjectModel | null = null
  public isEditing = false
  public isLoading = true
  public isMemoryFileLoading = false
  public isMemoryLoading = true
  public isMemorySearchLoading = false
  public isPlanFileLoading = false
  public isPlansLoading = true
  public isSaving = false
  public errorMessage: string | null = null
  public memoryErrorMessage: string | null = null
  public memoryFile: ProjectMemoryReadResult | null = null
  public memoryFileErrorMessage: string | null = null
  public memorySearchErrorMessage: string | null = null
  public memorySearchQuery = ''
  public memorySearchResults: ProjectMemorySearchResult[] = []
  public memoryTree: ProjectMemoryListResult | null = null
  public planFile: ProjectPlanReadResult | null = null
  public planFileErrorMessage: string | null = null
  public plansErrorMessage: string | null = null
  public plansList: ProjectPlansListResult | null = null
  public saveState: 'saved' | 'saving' | 'pending' | 'error' = 'saved'
  public selectedPlanPath: string | null = null
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

  public get hasMemorySearchQuery() {
    return this.memorySearchQuery.trim().length > 0
  }

  public get hasMemorySearchResults() {
    return this.memorySearchResults.length > 0
  }

  public get plans() {
    return this.plansList?.plans ?? []
  }

  public get hasPlans() {
    return this.plans.length > 0
  }

  public get selectedPlan() {
    return this.plans.find((plan) => plan.path === this.selectedPlanPath) ?? null
  }

  public get selectedPlanFileName() {
    return this.selectedPlan?.filename ?? this.selectedPlanPath?.split('/').at(-1) ?? null
  }

  public get selectedPlanContent() {
    return this.planFile?.content ?? ''
  }

  public get selectedPlanFileType() {
    return getPlanFileType(this.planFile)
  }

  public get canPreviewSelectedPlanFile() {
    return this.planFile?.is_previewable ?? false
  }

  public setActiveTab(value: 'overview' | 'memory' | 'plans') {
    this.activeTab = value
  }

  public setMemorySearchQuery(value: string) {
    this.memorySearchQuery = value

    if (value.trim().length === 0) {
      this.memorySearchResults = []
      this.memorySearchErrorMessage = null
      this.isMemorySearchLoading = false
    }
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

    const [projectResult, memoryResult, plansResult] = await Promise.allSettled([
      projectsApi.get(this.projectId),
      projectsApi.memory(this.projectId),
      projectsApi.plans(this.projectId),
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

      if (plansResult.status === 'fulfilled') {
        this.applyPlansList(plansResult.value)
      } else {
        this.plansErrorMessage = getErrorMessage(plansResult.reason, 'Unable to load project plans.')
      }

      runInAction(() => {
        this.isLoading = false
        this.isMemoryLoading = false
        this.isPlansLoading = false
      })
    })

    if (this.selectedMemoryPath) {
      await this.selectMemoryPath(this.selectedMemoryPath)
    }

    if (this.selectedPlanPath) {
      await this.selectPlanPath(this.selectedPlanPath)
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

  public async reloadPlans() {
    runInAction(() => {
      this.isPlansLoading = true
      this.plansErrorMessage = null
    })

    try {
      const plans = await projectsApi.plans(this.projectId)

      runInAction(() => {
        this.applyPlansList(plans)
      })

      if (this.selectedPlanPath) {
        await this.selectPlanPath(this.selectedPlanPath)
      }
    } catch (error) {
      runInAction(() => {
        this.plansErrorMessage = getErrorMessage(error, 'Unable to load project plans.')
      })
    } finally {
      runInAction(() => {
        this.isPlansLoading = false
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

  public async searchMemory() {
    const query = this.memorySearchQuery.trim()

    if (!query) {
      runInAction(() => {
        this.memorySearchResults = []
        this.memorySearchErrorMessage = null
        this.isMemorySearchLoading = false
      })
      return
    }

    runInAction(() => {
      this.isMemorySearchLoading = true
      this.memorySearchErrorMessage = null
    })

    try {
      const results = await projectsApi.searchMemory(this.projectId, query)

      runInAction(() => {
        if (this.memorySearchQuery.trim() !== query) return
        this.memorySearchResults = results
      })
    } catch (error) {
      runInAction(() => {
        if (this.memorySearchQuery.trim() !== query) return
        this.memorySearchResults = []
        this.memorySearchErrorMessage = getErrorMessage(error, 'Unable to search project memory.')
      })
    } finally {
      runInAction(() => {
        if (this.memorySearchQuery.trim() !== query) return
        this.isMemorySearchLoading = false
      })
    }
  }

  public async selectMemorySearchResult(result: ProjectMemorySearchResult) {
    if (!result.path) {
      runInAction(() => {
        this.memoryFileErrorMessage = 'This search result does not include a file path to open.'
      })
      return
    }

    await this.selectMemoryPath(result.path)
  }

  public async selectPlanPath(path: string) {
    runInAction(() => {
      this.selectedPlanPath = path
      this.isPlanFileLoading = true
      this.planFileErrorMessage = null
    })

    try {
      const file = await projectsApi.readPlanFile(this.projectId, path)

      runInAction(() => {
        if (this.selectedPlanPath !== path) return
        this.planFile = file
      })
    } catch (error) {
      runInAction(() => {
        if (this.selectedPlanPath !== path) return
        this.planFile = null
        this.planFileErrorMessage = getErrorMessage(error, 'Unable to load this plan file.')
      })
    } finally {
      runInAction(() => {
        if (this.selectedPlanPath !== path) return
        this.isPlanFileLoading = false
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

  private applyPlansList(plansList: ProjectPlansListResult) {
    const availablePaths = plansList.plans.map((plan) => plan.path)
    const selectedPath = selectDefaultPlanPath(plansList.plans)

    this.plansList = plansList
    this.plansErrorMessage = null

    if (availablePaths.length === 0) {
      this.selectedPlanPath = null
      this.planFile = null
      this.planFileErrorMessage = null
      return
    }

    if (this.selectedPlanPath && availablePaths.includes(this.selectedPlanPath)) return

    this.selectedPlanPath = selectedPath
    this.planFile = null
    this.planFileErrorMessage = null
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

export const getProjectMemoryResultLabel = (result: ProjectMemorySearchResult) => {
  return result.path?.split('/').at(-1) ?? result.title ?? 'Untitled result'
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

const selectDefaultPlanPath = (plans: ProjectPlanListItem[]) => {
  return plans.find((plan) => !plan.is_superseded)?.path ?? plans[0]?.path ?? null
}

const getMemoryFileType = (path: string | null) => {
  const normalizedPath = path?.toLowerCase() ?? ''

  if (normalizedPath.endsWith('.md') || normalizedPath.endsWith('.markdown')) return 'markdown'
  if (
    normalizedPath.endsWith('.txt') ||
    normalizedPath.endsWith('.log') ||
    normalizedPath.endsWith('.json') ||
    normalizedPath.endsWith('.yaml') ||
    normalizedPath.endsWith('.yml')
  ) {
    return 'text'
  }

  return 'unsupported'
}

const getPlanFileType = (file: ProjectPlanReadResult | null) => {
  if (!file?.is_previewable) return 'unsupported'
  if (file.mime_type === 'text/markdown') return 'markdown'
  if (file.mime_type.startsWith('text/')) return 'text'

  return 'unsupported'
}
