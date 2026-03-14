import { makeAutoObservable, runInAction } from 'mobx'
import type { PlanDocumentStatus, PlanListQuery, PlanListSortBy, ProjectPlanListItem, SortDirection } from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
// eslint-disable-next-line
import { tauriProjectApi } from '@/lib/api/tauri/project.api'
import { PlanModel } from '@/lib/models/plan.model'

export type PlanSortOption = 'updated_desc' | 'updated_asc' | 'created_desc' | 'created_asc' | 'name_asc' | 'name_desc'
export type PlanStatusFilter = 'all' | PlanDocumentStatus

const DEFAULT_SORT: PlanSortOption = 'updated_desc'
const SEARCH_DEBOUNCE_MS = 250

const parseSortOption = (sortOption: PlanSortOption): { by: PlanListSortBy; direction: SortDirection } => {
  switch (sortOption) {
    case 'updated_asc':
      return { by: 'updated_at', direction: 'asc' }
    case 'created_desc':
      return { by: 'created_at', direction: 'desc' }
    case 'created_asc':
      return { by: 'created_at', direction: 'asc' }
    case 'name_asc':
      return { by: 'name', direction: 'asc' }
    case 'name_desc':
      return { by: 'name', direction: 'desc' }
    default:
      return { by: 'updated_at', direction: 'desc' }
  }
}

const defaultStatuses: PlanDocumentStatus[] = ['pending', 'in_progress', 'completed', 'archived']

export class ProjectPlansListViewModel {
  public isLoading = false
  public error: string | null = null
  public plans: ProjectPlanListItem[] = []
  public search = ''
  public sort: PlanSortOption = DEFAULT_SORT
  public statusFilter: PlanStatusFilter = 'pending'
  public selectedPlanIds = new Set<string>()
  public bulkStatus: PlanDocumentStatus | null = null
  public isBulkUpdating = false
  private searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

  constructor(private readonly projectId: string) {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = async () => {
    await this.load()
  }

  destroy = () => {
    if (this.searchDebounceTimer) {
      clearTimeout(this.searchDebounceTimer)
      this.searchDebounceTimer = null
    }
  }

  setSearch = (search: string) => {
    this.search = search
    if (this.searchDebounceTimer) {
      clearTimeout(this.searchDebounceTimer)
    }
    this.searchDebounceTimer = setTimeout(() => {
      void this.load()
    }, SEARCH_DEBOUNCE_MS)
  }

  setSort = (sort: PlanSortOption) => {
    this.sort = sort
    void this.load()
  }

  setStatusFilter = (statusFilter: PlanStatusFilter) => {
    this.statusFilter = statusFilter
    void this.load()
  }

  setBulkStatus = (status: PlanDocumentStatus | null) => {
    this.bulkStatus = status
  }

  togglePlanSelected = (planId: string) => {
    if (this.selectedPlanIds.has(planId)) {
      this.selectedPlanIds.delete(planId)
    } else {
      this.selectedPlanIds.add(planId)
    }
  }

  toggleSelectAll = () => {
    if (this.isAllSelected) {
      this.selectedPlanIds.clear()
      return
    }

    this.plans.forEach((plan) => {
      this.selectedPlanIds.add(plan.id)
    })
  }

  clearSelection = () => {
    this.selectedPlanIds.clear()
  }

  get selectedCount() {
    return this.selectedPlanIds.size
  }

  get isAllSelected() {
    return this.plans.length > 0 && this.selectedPlanIds.size === this.plans.length
  }

  get isSelectionIndeterminate() {
    return this.selectedPlanIds.size > 0 && this.selectedPlanIds.size < this.plans.length
  }

  get statusOptions() {
    return defaultStatuses
  }

  getStatusLabel = (status: PlanDocumentStatus) => {
    switch (status) {
      case 'pending':
        return 'Pending'
      case 'in_progress':
        return 'In Progress'
      case 'completed':
        return 'Completed'
      case 'archived':
        return 'Archived'
    }
  }

  updatePlanStatus = async (planId: string, status: PlanDocumentStatus) => {
    const target = this.plans.find((plan) => plan.id === planId)
    const previousStatus = target?.status ?? 'pending'

    runInAction(() => {
      if (target) target.status = status
    })

    try {
      await tauriProjectApi.planUpdate(this.projectId, planId, { status })
    } catch (error) {
      runInAction(() => {
        if (target) target.status = previousStatus
      })
      const message = error instanceof Error ? error.message : 'Failed to update plan status'
      basicToast.error({ description: message, title: 'Failed to update status' })
    }
  }

  applyBulkStatus = async () => {
    if (!this.bulkStatus || this.selectedPlanIds.size === 0) return
    this.isBulkUpdating = true

    const targetStatus = this.bulkStatus
    const selectedIds = Array.from(this.selectedPlanIds)
    const previousStatuses = new Map<string, PlanDocumentStatus>()

    runInAction(() => {
      selectedIds.forEach((planId) => {
        const plan = this.plans.find((p) => p.id === planId)
        if (!plan) return
        previousStatuses.set(planId, plan.status ?? 'pending')
        plan.status = targetStatus
      })
    })

    try {
      const results = await Promise.allSettled(
        selectedIds.map((planId) => tauriProjectApi.planUpdate(this.projectId, planId, { status: targetStatus })),
      )

      const failures: { planId: string; message: string }[] = []
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') return
        const planId = selectedIds[index]
        const message = result.reason instanceof Error ? result.reason.message : 'Failed to update'
        failures.push({ message, planId })
      })

      if (failures.length > 0) {
        runInAction(() => {
          failures.forEach(({ planId }) => {
            const plan = this.plans.find((p) => p.id === planId)
            const previousStatus = previousStatuses.get(planId)
            if (plan && previousStatus) plan.status = previousStatus
          })
        })

        const summary =
          failures.length === 1
            ? failures[0]?.message
            : `${failures.length} plans failed to update. First error: ${failures[0]?.message}`
        basicToast.error({ description: summary, title: 'Bulk status update failed' })
        return
      }

      runInAction(() => {
        this.clearSelection()
        this.bulkStatus = null
      })
    } finally {
      runInAction(() => {
        this.isBulkUpdating = false
      })
    }
  }

  private buildQuery = (): PlanListQuery => {
    const trimmedSearch = this.search.trim()

    const statusFilter = this.statusFilter === 'all' ? defaultStatuses : [this.statusFilter]

    return {
      search: trimmedSearch.length > 0 ? trimmedSearch : null,
      sort: parseSortOption(this.sort),
      status_filter: statusFilter,
    }
  }

  load = async () => {
    this.isLoading = true
    this.error = null
    try {
      const query = this.buildQuery()
      const result = await PlanModel.listForProject(this.projectId, query)
      runInAction(() => {
        this.plans = result
        this.selectedPlanIds.forEach((planId) => {
          if (!result.some((plan) => plan.id === planId)) {
            this.selectedPlanIds.delete(planId)
          }
        })
      })
    } catch (error) {
      runInAction(() => {
        this.error = error instanceof Error ? error.message : 'Failed to load plans'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }
}
