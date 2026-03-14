import { debounce } from 'lodash'
import { flow, makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import type { LlmEvent, PlanDocumentStatus, PlanTodoItem, ToolUseResponseData } from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'

import { EventType, globalEventBus } from '@/lib/events'
import { PlanModel } from '@/lib/models/plan.model'

type SaveState = 'idle' | 'saving' | 'saved' | 'error'

export class PlanPanelViewmodel {
  public plan: PlanModel | null = null
  public isLoading = false
  public error: string | null = null
  public draftContent = ''
  public draftTodos: PlanTodoItem[] = []
  public saveState: SaveState = 'idle'
  public isContentEditing = false
  public editingTodoId: string | null = null
  public editingTodoContent = ''
  public isAddingTodo = false
  public addingTodoContent = ''
  public resolvedPlanStatus: PlanDocumentStatus | null = null
  private hasPendingChanges = false
  private isSaving = false
  private saveVersion = 0
  private lastSavedVersion = 0
  private unsubscribers: Array<() => void> = []
  private readonly debouncedSave = debounce(() => {
    void this.savePlan()
  }, 1000)
  private isLoaded = false

  constructor(
    private readonly projectId: string,
    private readonly planId: string,
  ) {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = flow(function* (this: PlanPanelViewmodel) {
    yield this.load()
    this.startListening()
    setTimeout(() => (this.isLoaded = true), 1000)
  })

  destroy = () => {
    this.unsubscribers.forEach((unsubscribe) => unsubscribe())
    this.unsubscribers = []
  }

  get isReadOnly() {
    return this.plan?.status !== 'pending'
  }

  get saveStatusLabel() {
    if (this.isReadOnly) return 'read-only'
    if (this.saveState === 'saving') return 'saving...'
    if (this.saveState === 'saved') return 'saved'
    if (this.saveState === 'error') return 'save failed'
    return ''
  }

  setContentEditing = (value: boolean) => {
    if (this.isReadOnly) return
    this.isContentEditing = value
  }

  updateDraftContent = (content: string) => {
    if (this.isReadOnly || !this.isLoaded) return
    this.draftContent = content
    this.scheduleSave()
  }

  startAddTodo = () => {
    if (this.isReadOnly) return
    this.isAddingTodo = true
    this.addingTodoContent = ''
  }

  updateAddingTodoContent = (content: string) => {
    if (this.isReadOnly) return
    this.addingTodoContent = content
  }

  cancelAddTodo = () => {
    this.isAddingTodo = false
    this.addingTodoContent = ''
  }

  confirmAddTodo = () => {
    if (this.isReadOnly) return
    const content = this.addingTodoContent.trim()
    if (!content) {
      this.cancelAddTodo()
      return
    }
    this.draftTodos = [...this.draftTodos, { content, id: crypto.randomUUID(), status: 'pending' }]
    this.cancelAddTodo()
    this.scheduleSave()
  }

  startEditTodo = (todoId: string) => {
    if (this.isReadOnly) return
    const todo = this.draftTodos.find((item) => item.id === todoId)
    if (!todo) return
    this.editingTodoId = todoId
    this.editingTodoContent = todo.content
  }

  updateEditingTodoContent = (content: string) => {
    if (this.isReadOnly) return
    this.editingTodoContent = content
  }

  cancelEditTodo = () => {
    this.editingTodoId = null
    this.editingTodoContent = ''
  }

  confirmEditTodo = () => {
    if (this.isReadOnly || !this.editingTodoId) return
    const content = this.editingTodoContent.trim()
    if (!content) {
      this.deleteTodo(this.editingTodoId)
      this.cancelEditTodo()
      return
    }

    this.draftTodos = this.draftTodos.map((todo) =>
      todo.id === this.editingTodoId
        ? {
            ...todo,
            content,
          }
        : todo,
    )
    this.cancelEditTodo()
    this.scheduleSave()
  }

  deleteTodo = (todoId: string) => {
    if (this.isReadOnly) return
    this.draftTodos = this.draftTodos.filter((todo) => todo.id !== todoId)
    if (this.editingTodoId === todoId) {
      this.cancelEditTodo()
    }
    this.scheduleSave()
  }

  private scheduleSave = () => {
    if (!this.plan || this.isReadOnly) return
    this.hasPendingChanges = true
    this.saveVersion += 1
    this.saveState = 'saving'
    this.debouncedSave()
  }

  private savePlan = flow(function* (this: PlanPanelViewmodel) {
    if (!this.plan || this.isReadOnly || this.isSaving || !this.hasPendingChanges) return

    this.isSaving = true
    const currentVersion = this.saveVersion

    try {
      yield this.plan.update({
        content: this.draftContent,
        todos: this.draftTodos,
      })

      this.lastSavedVersion = currentVersion
      this.hasPendingChanges = this.lastSavedVersion < this.saveVersion
      this.syncDraftsFromPlan()
      this.saveState = this.hasPendingChanges ? 'saving' : 'saved'

      if (this.hasPendingChanges) {
        this.debouncedSave()
      }
    } catch (error) {
      this.saveState = 'error'
      this.hasPendingChanges = this.lastSavedVersion < this.saveVersion

      const errorMessage = String(error)
      if (this.isPlanLockedError(errorMessage)) {
        this.hasPendingChanges = false
        yield this.reloadCanonical()
      }

      basicToast.error({ description: String(error), title: 'Failed to save plan updates' })
    } finally {
      this.isSaving = false
    }
  })

  private isPlanLockedError = (errorMessage: string) => {
    const normalized = errorMessage.toLowerCase()
    return (
      normalized.includes('pending') ||
      normalized.includes('only pending') ||
      normalized.includes('not editable') ||
      normalized.includes('cannot update plan')
    )
  }

  private reloadCanonical = flow(function* (this: PlanPanelViewmodel) {
    this.plan = yield PlanModel.get(this.projectId, this.planId)
    this.syncDraftsFromPlan()
  })

  private syncDraftsFromPlan = () => {
    if (!this.plan) return
    this.draftContent = this.plan.content
    this.draftTodos = [...this.plan.todos]
  }

  private startListening = () => {
    this.unsubscribers.push(
      globalEventBus.subscribe(
        EventType.SessionLlm,
        () => {
          void this.load()
        },
        (event) => this.isPlanUpdateEvent(event.payload.event),
      ),
    )
  }

  private isPlanUpdateEvent = (event: LlmEvent) => {
    if (!event || event.type !== 'toolCallCompleted') return false
    if (event.content.success !== true || event.content.type !== 'success') return false
    const data = event.content.data as ToolUseResponseData
    if (data.type !== 'plan_update') return false
    if (data.id !== this.plan?.id) return false

    return true
  }

  private load = flow(function* (this: PlanPanelViewmodel) {
    this.isLoading = true
    this.error = null

    try {
      this.plan = yield PlanModel.get(this.projectId, this.planId)
      this.syncDraftsFromPlan()
    } catch (error) {
      this.error = String(error)
    } finally {
      this.isLoading = false
    }
  })
}

export const PlanPanelViewmodelContext = createContext<PlanPanelViewmodel | null>(null)

export const usePlanPanelViewmodel = () => {
  const context = useContext(PlanPanelViewmodelContext)
  if (!context) throw new Error('PlanPanelViewmodelContext is not available')
  return context
}
