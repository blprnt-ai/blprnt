import { Check, Circle, CircleCheckBig, LucideCircleDotDashed, Pencil, Plus, Trash2, X } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import Markdown from 'react-markdown'
import { Button } from '@/components/atoms/button'
import { Input } from '@/components/atoms/input'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { cn } from '@/lib/utils/cn'
import { usePlanPanelViewmodel } from './plan-panel.viewmodel'

export const PlanPanel = observer(() => {
  const viewmodel = usePlanPanelViewmodel()

  if (viewmodel.isLoading && !viewmodel.plan) {
    return <div className="p-4 text-sm text-muted-foreground">Loading plan...</div>
  }

  if (viewmodel.error && !viewmodel.plan) {
    return <div className="p-4 text-sm text-destructive">Failed to load plan: {viewmodel.error}</div>
  }

  if (!viewmodel.plan) return null

  const plan = viewmodel.plan

  return (
    <div className="flex h-full w-full flex-col gap-4 overflow-hidden p-4 bg-gradient-glow">
      <header className="flex flex-col gap-1">
        <div className="flex items-center justify-between gap-2">
          <div className="text-lg font-semibold">{plan.name}</div>
          <div
            className={cn(
              'text-xs uppercase tracking-wide text-muted-foreground transition-colors',
              viewmodel.saveState === 'error' && 'text-destructive',
              viewmodel.saveState === 'saved' && 'text-success',
            )}
          >
            {viewmodel.saveStatusLabel}
          </div>
        </div>
        {plan.description?.trim() ? <div className="text-sm text-muted-foreground">{plan.description}</div> : null}
      </header>

      {(viewmodel.draftTodos.length > 0 || !viewmodel.isReadOnly) && (
        <div className={cn('rounded-md border p-3 bg-background', viewmodel.isReadOnly && 'opacity-80')}>
          <div className="text-xs uppercase text-muted-foreground">Todos</div>
          <div className="overflow-y-auto max-h-[300px]">
            <div className="mt-2 flex flex-col gap-2">
              {viewmodel.draftTodos.map((todo) => {
                const isPending = todo.status === 'pending'
                const isCompleted = todo.status === 'complete'
                const inProgress = todo.status === 'in_progress'
                const isEditing = viewmodel.editingTodoId === todo.id
                return (
                  <div key={todo.id} className="flex items-center gap-2">
                    {isPending && <Circle className="size-4 text-muted-foreground shrink-0" />}
                    {inProgress && <LucideCircleDotDashed className="size-4 text-primary animate-spin shrink-0" />}
                    {isCompleted && <CircleCheckBig className="size-4 text-success shrink-0" />}

                    {isEditing ? (
                      <>
                        <Input
                          autoFocus
                          className="h-8"
                          value={viewmodel.editingTodoContent}
                          onBlur={viewmodel.confirmEditTodo}
                          onChange={(event) => viewmodel.updateEditingTodoContent(event.target.value)}
                          onKeyDown={(event) => {
                            if (event.key === 'Enter') {
                              event.preventDefault()
                              viewmodel.confirmEditTodo()
                            }
                            if (event.key === 'Escape') {
                              event.preventDefault()
                              viewmodel.cancelEditTodo()
                            }
                          }}
                        />
                        <Button size="icon-xs" type="button" variant="ghost" onClick={viewmodel.confirmEditTodo}>
                          <Check className="size-3.5" />
                        </Button>
                        <Button size="icon-xs" type="button" variant="ghost" onClick={viewmodel.cancelEditTodo}>
                          <X className="size-3.5" />
                        </Button>
                      </>
                    ) : (
                      <>
                        <div
                          className={cn(
                            'text-sm flex-1',
                            inProgress && 'animate-pulse text-primary',
                            isCompleted && 'line-through text-muted-foreground',
                          )}
                        >
                          {todo.content}
                        </div>
                        {!viewmodel.isReadOnly && (
                          <>
                            <Button
                              size="icon-xs"
                              type="button"
                              variant="ghost"
                              onClick={() => viewmodel.startEditTodo(todo.id)}
                            >
                              <Pencil className="size-3.5" />
                            </Button>
                            <Button
                              size="icon-xs"
                              type="button"
                              variant="ghost"
                              onClick={() => viewmodel.deleteTodo(todo.id)}
                            >
                              <Trash2 className="size-3.5" />
                            </Button>
                          </>
                        )}
                      </>
                    )}
                  </div>
                )
              })}

              {!viewmodel.isReadOnly && !viewmodel.isAddingTodo && (
                <Button className="w-fit px-0" size="sm" type="button" variant="link" onClick={viewmodel.startAddTodo}>
                  <Plus className="size-3.5" />
                  Add todo
                </Button>
              )}

              {!viewmodel.isReadOnly && viewmodel.isAddingTodo && (
                <div className="flex items-center gap-2">
                  <Circle className="size-4 text-muted-foreground shrink-0" />
                  <Input
                    autoFocus
                    className="h-8"
                    placeholder="Todo"
                    value={viewmodel.addingTodoContent}
                    onBlur={viewmodel.confirmAddTodo}
                    onChange={(event) => viewmodel.updateAddingTodoContent(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter') {
                        event.preventDefault()
                        viewmodel.confirmAddTodo()
                      }
                      if (event.key === 'Escape') {
                        event.preventDefault()
                        viewmodel.cancelAddTodo()
                      }
                    }}
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      <div
        className={cn(
          'min-h-0 h-full flex-1 overflow-auto',
          viewmodel.isReadOnly && 'rounded-md border bg-background p-4',
        )}
      >
        {!viewmodel.isReadOnly ? (
          <MarkdownEditor
            className="min-h-full border-none bg-transparent px-0 py-0 focus-visible:ring-0"
            value={viewmodel.draftContent}
            onChange={(value) => viewmodel.updateDraftContent(value)}
          />
        ) : (
          <Markdown>{viewmodel.draftContent}</Markdown>
        )}
      </div>
    </div>
  )
})
