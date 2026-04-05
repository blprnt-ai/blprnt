import { type IReactionDisposer, makeAutoObservable, reaction, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeLifeFileResult } from '@/bindings/EmployeeLifeFileResult'
import type { EmployeeLifeTreeNode } from '@/bindings/EmployeeLifeTreeNode'
import type { EmployeeLifeTreeResult } from '@/bindings/EmployeeLifeTreeResult'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { IssueDto } from '@/bindings/IssueDto'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import type { Skill } from '@/bindings/Skill'
import { IssueFormViewmodel } from '@/components/forms/issue/issue-form.viewmodel'
import { employeesApi } from '@/lib/api/employees'
import { providersApi } from '@/lib/api/providers'
import { runsApi } from '@/lib/api/runs'
import { skillsApi } from '@/lib/api/skills'
import { AppModel } from '@/models/app.model'
import { EmployeeModel } from '@/models/employee.model'
import type { RunsViewmodel } from '@/runs.viewmodel'
import { getRuntimeProviderOptions } from './utils'

export class EmployeeViewmodel {
  public activeTab: 'profile' | 'runtime' | 'life' = 'profile'
  public availableSkills: Skill[] = []
  public configuredProviders: ProviderDto[] = []
  public employee: EmployeeModel | null = null
  public isEditing = false
  public isConfiguredProvidersLoaded = false
  public isLoading = true
  public isLifeFileLoading = false
  public isLifeLoading = false
  public isLifeSaving = false
  public isSaving = false
  public isSkillsLoading = false
  public isStatusUpdating = false
  public isTerminating = false
  public isTriggeringRun = false
  public errorMessage: string | null = null
  public lifeDraftContent = ''
  public lifeErrorMessage: string | null = null
  public lifeFile: EmployeeLifeFileResult | null = null
  public lifeTree: EmployeeLifeTreeResult | null = null
  public skillsErrorMessage: string | null = null
  public saveState: 'saved' | 'saving' | 'pending' | 'error' = 'saved'
  public lastSavedAt: Date | null = null
  public selectedLifePath: string | null = null
  public readonly issueFormViewmodel: IssueFormViewmodel
  private readonly employeeId: string
  private readonly runs?: RunsViewmodel
  private readonly onTerminated?: () => Promise<void> | void
  private readonly onOpenChat?: (employeeId: string) => Promise<void> | void
  private readonly onRunCreated?: (runId: string) => Promise<void> | void
  private originalEmployee: Employee | null = null
  private autosaveTimer: ReturnType<typeof setTimeout> | null = null
  private autosaveDisposer: IReactionDisposer | null = null
  private saveQueued = false
  private readonly autosaveDelayMs = 800

  constructor(
    employeeId: string,
    options?: {
      onIssueCreated?: (issue: IssueDto) => Promise<void> | void
      onOpenChat?: (employeeId: string) => Promise<void> | void
      onRunCreated?: (runId: string) => Promise<void> | void
      onTerminated?: () => Promise<void> | void
      runs?: RunsViewmodel
    },
  ) {
    this.employeeId = employeeId
    this.onOpenChat = options?.onOpenChat
    this.onRunCreated = options?.onRunCreated
    this.onTerminated = options?.onTerminated
    this.runs = options?.runs
    this.issueFormViewmodel = new IssueFormViewmodel(options?.onIssueCreated)

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get canSave() {
    return Boolean(this.employee?.isDirty) && !this.isSaving
  }

  public get saveStatusLabel() {
    switch (this.saveState) {
      case 'saving':
        return 'Saving changes...'
      case 'pending':
        return 'Changes pending'
      case 'error':
        return 'Autosave failed'
      default:
        if (!this.lastSavedAt) return 'Ready'

        return `Saved ${formatTime(this.lastSavedAt)}`
    }
  }

  public get saveStatusHint() {
    switch (this.saveState) {
      case 'saving':
        return 'Updates are being written now.'
      case 'pending':
        return 'Keep editing. Everything saves automatically.'
      case 'error':
        return this.errorMessage ?? 'We could not save your latest changes.'
      default:
        return 'This page saves in place as you edit.'
    }
  }

  public get capabilitiesValue() {
    return this.employee?.capabilities.join(', ') ?? ''
  }

  public get showsAgentConfiguration() {
    return this.employee?.kind === 'agent'
  }

  public get isHumanEmployee() {
    return this.employee?.kind === 'person'
  }

  public get canEditSelectedLifeFile() {
    return Boolean(this.lifeFile?.editable)
  }

  public get hasUnsavedLifeChanges() {
    return this.lifeDraftContent !== (this.lifeFile?.content ?? '')
  }

  public get isOwnerEmployee() {
    return this.employee?.role === 'owner'
  }

  public get isPaused() {
    return this.employee?.status === 'paused'
  }

  public get isTerminated() {
    return this.employee?.status === 'terminated'
  }

  public get canTriggerRun() {
    return (
      Boolean(this.employee?.id) &&
      this.showsAgentConfiguration &&
      !this.isPaused &&
      !this.isTerminated &&
      !this.isTriggeringRun &&
      !this.isTerminating
    )
  }

  public get pauseResumeLabel() {
    return this.isPaused ? 'Resume' : 'Pause'
  }

  public get pauseResumePendingLabel() {
    return this.isPaused ? 'Resuming...' : 'Pausing...'
  }

  public get triggerRunLabel() {
    return this.isTriggeringRun ? 'Starting...' : 'Run now'
  }

  public get roleValue() {
    if (!this.employee) return ''
    if (typeof this.employee.role === 'string') return this.employee.role
    if ('custom' in this.employee.role) return this.employee.role.custom

    return ''
  }

  public get reportsTo() {
    return this.employee?.reports_to ?? this.originalEmployee?.reports_to ?? null
  }

  public get chainOfCommand() {
    return this.originalEmployee?.chain_of_command ?? []
  }

  public get lifeTreeNodes() {
    return this.lifeTree?.nodes ?? []
  }

  public get runtimeProviderOptions() {
    return getRuntimeProviderOptions({
      configuredProviders: this.configuredProviders,
      currentProvider: this.employee?.provider ?? 'claude_code',
      disableUnconfiguredProviders: this.isConfiguredProvidersLoaded,
    })
  }

  public async init() {
    runInAction(() => {
      this.configuredProviders = []
      this.isLoading = true
      this.isConfiguredProvidersLoaded = false
      this.isLifeLoading = true
      this.isSkillsLoading = true
      this.errorMessage = null
      this.lifeErrorMessage = null
      this.skillsErrorMessage = null
    })

    const [employeeResult, lifeResult, skillsResult, providersResult] = await Promise.allSettled([
      employeesApi.get(this.employeeId),
      employeesApi.life(this.employeeId),
      skillsApi.list(),
      providersApi.list(),
    ])

    runInAction(() => {
      if (employeeResult.status === 'fulfilled') {
        this.setEmployee(employeeResult.value)
      } else {
        this.errorMessage = getErrorMessage(employeeResult.reason, 'Unable to load this employee.')
      }

      if (lifeResult.status === 'fulfilled' && Array.isArray(lifeResult.value?.nodes)) {
        this.lifeTree = lifeResult.value
        this.selectedLifePath = getDefaultLifePath(lifeResult.value.nodes)
      } else if (lifeResult.status === 'fulfilled') {
        this.lifeTree = { nodes: [], root_path: '$AGENT_HOME' }
        this.selectedLifePath = null
      } else {
        this.lifeErrorMessage = getErrorMessage(lifeResult.reason, 'Unable to load this employee life.')
      }

      if (skillsResult.status === 'fulfilled') {
        this.availableSkills = skillsResult.value
      } else {
        this.skillsErrorMessage = getErrorMessage(skillsResult.reason, 'Unable to load available skills.')
      }

      if (providersResult.status === 'fulfilled') {
        this.configuredProviders = providersResult.value
        this.isConfiguredProvidersLoaded = true
      }

      this.isLoading = false
      this.isLifeLoading = false
      this.isSkillsLoading = false
    })

    if (this.selectedLifePath) {
      await this.selectLifePath(this.selectedLifePath)
    }
  }

  public async save() {
    if (this.isSaving) {
      this.saveQueued = true
      return this.employee
    }

    if (!this.employee?.id || !this.employee.isDirty) {
      runInAction(() => {
        if (this.saveState !== 'error') this.saveState = 'saved'
      })
      return this.employee
    }

    runInAction(() => {
      this.isSaving = true
      this.errorMessage = null
      this.saveState = 'saving'
    })

    try {
      const employee = await employeesApi.update(this.employee.id, this.employee.toPayloadPatch())

      runInAction(() => {
        this.setEmployee(employee)
        this.lastSavedAt = new Date()
        this.saveState = 'saved'
      })

      return this.employee
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to save this employee.')
        this.saveState = 'error'
      })

      return null
    } finally {
      runInAction(() => {
        this.isSaving = false
      })

      if (this.saveQueued || this.employee?.isDirty) {
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
    this.issueFormViewmodel.cancel()
  }

  public setActiveTab(value: 'profile' | 'runtime' | 'life') {
    this.activeTab = value
  }

  public startEditing() {
    if (!this.employee) return

    this.isEditing = true
  }

  public cancelEditing() {
    if (!this.originalEmployee) return

    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer)
      this.autosaveTimer = null
    }

    this.saveQueued = false
    this.errorMessage = null
    this.saveState = 'saved'
    this.setEmployee(this.originalEmployee)
  }

  public setCapabilities(value: string) {
    if (!this.employee) return

    this.employee.capabilities = value
      .split(',')
      .map((part) => part.trim())
      .filter(Boolean)
  }

  public setRole(value: string) {
    if (!this.employee) return

    this.employee.role = parseRole(value)
  }

  public setProvider(value: Provider) {
    if (!this.employee) return

    this.employee.provider = value
  }

  public setSlug(value: string) {
    if (!this.employee) return

    this.employee.slug = value
  }

  public setSkillAt(index: number, skill: Skill | null) {
    if (!this.employee) return

    const nextSkills = [...this.employee.skill_stack]
    if (skill) {
      nextSkills[index] = { name: skill.name, path: skill.path }
    } else {
      nextSkills.splice(index, 1)
    }

    this.employee.skill_stack = nextSkills
  }

  public async selectLifePath(path: string) {
    if (this.selectedLifePath === path && this.lifeFile?.path === path && !this.lifeErrorMessage) return

    runInAction(() => {
      this.isLifeFileLoading = true
      this.lifeErrorMessage = null
      this.selectedLifePath = path
    })

    try {
      const file = await employeesApi.readLifeFile(this.employeeId, path)
      runInAction(() => {
        this.lifeFile = file
        this.lifeDraftContent = file.content
      })
    } catch (error) {
      runInAction(() => {
        this.lifeErrorMessage = getErrorMessage(error, 'Unable to load this file.')
        this.lifeFile = null
        this.lifeDraftContent = ''
      })
    } finally {
      runInAction(() => {
        this.isLifeFileLoading = false
      })
    }
  }

  public setLifeDraftContent(value: string) {
    this.lifeDraftContent = value
  }

  public async saveLifeFile() {
    if (!this.selectedLifePath || !this.canEditSelectedLifeFile || this.isLifeSaving) return null

    runInAction(() => {
      this.isLifeSaving = true
      this.lifeErrorMessage = null
    })

    try {
      const file = await employeesApi.updateLifeFile(this.employeeId, {
        content: this.lifeDraftContent,
        path: this.selectedLifePath,
      })

      runInAction(() => {
        this.lifeFile = file
        this.lifeDraftContent = file.content
        this.refreshLifeTreeFile(file.path, file.editable)
      })

      return file
    } catch (error) {
      runInAction(() => {
        this.lifeErrorMessage = getErrorMessage(error, 'Unable to save this file.')
      })

      return null
    } finally {
      runInAction(() => {
        this.isLifeSaving = false
      })
    }
  }

  public openAddIssue() {
    if (!this.employee?.id || !this.showsAgentConfiguration || this.isTerminated) return

    this.issueFormViewmodel.openWithDefaults({ assignee: this.employee.id })
  }

  public async openChat() {
    if (!this.employee?.id || !this.showsAgentConfiguration || this.isTerminated) return
    await this.onOpenChat?.(this.employee.id)
  }

  public async triggerRun() {
    if (!this.employee?.id || !this.canTriggerRun) return false

    if (this.employee.isDirty) {
      await this.save()
      if (!this.employee?.id) return false
    }

    runInAction(() => {
      this.isTriggeringRun = true
      this.errorMessage = null
    })

    try {
      const run = await runsApi.trigger({
        employee_id: this.employee.id,
        prompt: null,
        reasoning_effort: null,
        trigger: 'manual',
      })

      this.runs?.upsertRun(run)
      await this.onRunCreated?.(run.id)
      return true
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to start a run for this employee.')
      })

      return false
    } finally {
      runInAction(() => {
        this.isTriggeringRun = false
      })
    }
  }

  public async togglePaused() {
    if (
      !this.employee?.id ||
      !this.showsAgentConfiguration ||
      this.isTerminated ||
      this.isStatusUpdating ||
      this.isTerminating
    ) {
      return null
    }

    if (this.employee.isDirty) {
      await this.save()
      if (!this.employee?.id) return null
    }

    const nextStatus = this.isPaused ? 'idle' : 'paused'

    runInAction(() => {
      this.isStatusUpdating = true
      this.errorMessage = null
    })

    try {
      const employee = await employeesApi.update(this.employee.id, {
        capabilities: null,
        color: null,
        icon: null,
        last_run_at: null,
        name: null,
        provider_config: null,
        reports_to: null,
        role: null,
        runtime_config: null,
        status: nextStatus,
        title: null,
      })

      runInAction(() => {
        this.setEmployee(employee)
      })

      return this.employee
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, `Unable to ${this.isPaused ? 'resume' : 'pause'} this employee.`)
      })

      return null
    } finally {
      runInAction(() => {
        this.isStatusUpdating = false
      })
    }
  }

  public async terminate() {
    if (!this.employee?.id || !this.showsAgentConfiguration || this.isTerminating) return false

    if (this.employee.isDirty) {
      await this.save()
      if (!this.employee?.id) return false
    }

    runInAction(() => {
      this.isTerminating = true
      this.errorMessage = null
    })

    try {
      await employeesApi.delete(this.employee.id)

      runInAction(() => {
        AppModel.instance.removeEmployee(this.employeeId)
        this.employee = null
      })

      await this.onTerminated?.()
      return true
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to terminate this employee.')
      })

      return false
    } finally {
      runInAction(() => {
        this.isTerminating = false
      })
    }
  }

  private setEmployee(employee: Employee) {
    this.originalEmployee = employee
    this.employee = new EmployeeModel(employee)
    this.isEditing = false
    this.setupAutosave()

    if (AppModel.instance.owner?.id === employee.id) {
      AppModel.instance.setOwner(employee)
      return
    }

    AppModel.instance.upsertEmployee(employee)
  }

  private setupAutosave() {
    this.autosaveDisposer?.()
    this.autosaveDisposer = reaction(
      () => (this.employee?.isDirty ? JSON.stringify(this.employee.toPayloadPatch()) : ''),
      (payload) => {
        if (!payload) return

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

  private refreshLifeTreeFile(path: string, editable: boolean) {
    if (!this.lifeTree) return

    this.lifeTree = {
      ...this.lifeTree,
      nodes: updateLifeTreeNodeEditability(this.lifeTree.nodes, path, editable),
    }
  }
}

export const EmployeeViewmodelContext = createContext<EmployeeViewmodel | null>(null)

export const useEmployeeViewmodel = () => {
  const viewmodel = useContext(EmployeeViewmodelContext)
  if (!viewmodel) throw new Error('EmployeeViewmodel not found')

  return viewmodel
}

const parseRole = (value: string): EmployeeRole => {
  if (value === 'owner' || value === 'ceo' || value === 'manager' || value === 'staff') return value

  return { custom: value }
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}

const formatTime = (value: Date) =>
  value.toLocaleTimeString([], {
    hour: 'numeric',
    minute: '2-digit',
  })

const getDefaultLifePath = (nodes: EmployeeLifeTreeNode[]): string | null => {
  for (const preferred of ['HEARTBEAT.md', 'SOUL.md', 'AGENTS.md', 'TOOLS.md']) {
    if (findLifeFilePath(nodes, preferred)) return preferred
  }

  return findFirstLifeFile(nodes)
}

const findLifeFilePath = (nodes: EmployeeLifeTreeNode[], target: string): string | null => {
  for (const node of nodes) {
    if (node.type === 'file' && node.path === target) return node.path
    if (node.type === 'directory') {
      const nested = findLifeFilePath(node.children, target)
      if (nested) return nested
    }
  }

  return null
}

const findFirstLifeFile = (nodes: EmployeeLifeTreeNode[]): string | null => {
  for (const node of nodes) {
    if (node.type === 'file') return node.path
    if (node.type === 'directory') {
      const nested = findFirstLifeFile(node.children)
      if (nested) return nested
    }
  }

  return null
}

const updateLifeTreeNodeEditability = (
  nodes: EmployeeLifeTreeNode[],
  targetPath: string,
  editable: boolean,
): EmployeeLifeTreeNode[] =>
  nodes.map((node) => {
    if (node.type === 'directory') {
      return {
        ...node,
        children: updateLifeTreeNodeEditability(node.children, targetPath, editable),
      }
    }

    if (node.path !== targetPath) return node

    return {
      ...node,
      editable,
    }
  })
