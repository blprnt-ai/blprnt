import {
  DndContext,
  type DragEndEvent,
  type DragOverEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useRouter } from '@tanstack/react-router'
import dayjs from 'dayjs'
import { useEffect, useMemo, useRef, useState } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { IssueDto } from '@/bindings/IssueDto'
import type { IssueStatus } from '@/bindings/IssueStatus'
import { IssueBadge } from '../pages/issue/components/issue-badge'
import { IdentityLink } from '../molecules/indentity'
import { PriorityIcon } from '../molecules/priority-icon'
import { type ColorVariant, colors, fallbackColor } from '../ui/colors'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select'
import { StatusIcon } from './status-icon'
import { cn } from '@/lib/utils'

const boardStatuses = ['backlog', 'todo', 'in_progress', 'blocked', 'done', 'cancelled']

function statusLabel(status: string): string {
  return status.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

interface KanbanBoardProps {
  issues: IssueDto[]
  employees?: Employee[]
  liveIssueIds?: Set<string>
  selectedIssueIds: Set<string>
  onToggleIssueSelection: (id: string) => void
  onUpdateIssue: (id: string, status: IssueStatus) => void
}

function resolveEmployeeColor(color: string): ColorVariant {
  return colors.some((entry) => entry.color === color) ? (color as ColorVariant) : fallbackColor
}

function KanbanColumn({
  status,
  issues,
  employees,
  hasActiveSelection,
  liveIssueIds,
  onToggleIssueSelection,
  onUpdateIssue,
  selectedIssueIds,
}: {
  status: string
  issues: IssueDto[]
  employees?: Employee[]
  hasActiveSelection: boolean
  liveIssueIds?: Set<string>
  onToggleIssueSelection: (id: string) => void
  onUpdateIssue: (id: string, status: IssueStatus) => void
  selectedIssueIds: Set<string>
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status })

  return (
    <div className="flex w-full flex-col md:w-[260px] md:min-w-[260px] md:shrink-0">
      <div className="mb-1 flex items-center gap-2 px-2 py-2">
        <StatusIcon status={status} />
        <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{statusLabel(status)}</span>
        <span className="ml-auto text-xs tabular-nums text-muted-foreground/60">{issues.length}</span>
      </div>
      <div
        ref={setNodeRef}
        className={`min-h-[120px] flex-1 space-y-1 rounded-md p-1 transition-colors ${isOver ? 'bg-accent/40' : 'bg-muted/20'}`}
      >
        <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
          {issues.map((issue) => (
            <KanbanCard
              key={issue.id}
              employees={employees}
              hasActiveSelection={hasActiveSelection}
              isLive={liveIssueIds?.has(issue.id)}
              isSelected={selectedIssueIds.has(issue.id)}
              issue={issue}
              onToggleIssueSelection={onToggleIssueSelection}
              onUpdateIssue={onUpdateIssue}
            />
          ))}
        </SortableContext>
      </div>
    </div>
  )
}

function KanbanCard({
  issue,
  employees,
  hasActiveSelection,
  isLive,
  isOverlay,
  isSelected,
  onToggleIssueSelection,
  onUpdateIssue,
}: {
  issue: IssueDto
  employees?: Employee[]
  hasActiveSelection?: boolean
  isLive?: boolean
  isOverlay?: boolean
  isSelected?: boolean
  onToggleIssueSelection?: (id: string) => void
  onUpdateIssue?: (id: string, status: IssueStatus) => void
}) {
  const { navigate } = useRouter()
  const longPressTimerRef = useRef<number | null>(null)
  const longPressTriggeredRef = useRef(false)
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    data: { issue },
    id: issue.id,
  })

  useEffect(
    () => () => {
      if (longPressTimerRef.current !== null) {
        window.clearTimeout(longPressTimerRef.current)
      }
    },
    [],
  )

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  const getEmployee = (id: string | null) => {
    if (!id || !employees) return null
    return employees.find((a) => a.id === id)
  }

  const clearLongPressTimer = () => {
    if (longPressTimerRef.current !== null) {
      window.clearTimeout(longPressTimerRef.current)
      longPressTimerRef.current = null
    }
  }

  const startLongPressSelection = () => {
    if (!onToggleIssueSelection || hasActiveSelection) return
    clearLongPressTimer()
    longPressTimerRef.current = window.setTimeout(() => {
      longPressTriggeredRef.current = true
      onToggleIssueSelection(issue.id)
    }, 450)
  }

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (isDragging) e.preventDefault()
    if (longPressTriggeredRef.current) {
      longPressTriggeredRef.current = false
      e.preventDefault()
      return
    }

    if (hasActiveSelection && onToggleIssueSelection) {
      e.preventDefault()
      onToggleIssueSelection(issue.id)
      return
    }

    navigate({
      params: { issueId: issue.id },
      to: '/issues/$issueId',
    })
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className={`cursor-grab rounded-md border bg-card p-2.5 transition-shadow active:cursor-grabbing ${
        isDragging && !isOverlay ? 'opacity-30' : ''
      } ${
        isOverlay
          ? 'shadow-lg ring-1 ring-primary/20'
          : isSelected
            ? 'border-primary bg-accent/30 shadow-sm ring-1 ring-primary/30'
            : 'hover:shadow-sm'
      }`}
    >
      <div
        className="block text-inherit no-underline"
        onClick={handleClick}
        onPointerCancel={clearLongPressTimer}
        onPointerDown={startLongPressSelection}
        onPointerLeave={clearLongPressTimer}
        onPointerUp={clearLongPressTimer}
      >
        <div className="mb-1.5 flex items-start gap-1.5">
          <div className="flex items-center gap-1.5">

          {isLive ? (
            <div className="bg-green-500/80 size-3 rounded-full animate-pulse" />
          ) : null}
          <span className={cn("shrink-0 font-mono text-xs text-muted-foreground", isLive && 'text-green-500 animate-pulse' )}>{issue.identifier ?? issue.id.slice(0, 8)}</span>
          </div>
          {isSelected ? <span className="ml-auto text-[10px] font-semibold uppercase tracking-wide text-primary">Selected</span> : null}
        </div>
        <p className="mb-2 line-clamp-2 text-sm leading-snug">{issue.title}</p>
        {issue.labels.length > 0 ? (
          <div className="mb-2 flex flex-wrap gap-1">
            {issue.labels.slice(0, 3).map((label) => (
              <IssueBadge key={label.name} className="text-[10px]">
                {label.name}
              </IssueBadge>
            ))}
          </div>
        ) : null}
        <div className="flex items-center gap-2 md:hidden">
          <PriorityIcon priority={issue.priority} />
          <div className="min-w-0 flex-1">
            {issue.assignee &&
              (() => {
                const employee = getEmployee(issue.assignee)
                const name = employee?.name
                const icon = employee?.icon
                const color = employee ? resolveEmployeeColor(employee.color) : null

                return name && icon && color ? (
                  <IdentityLink color={color} employeeId={employee?.id} icon={icon} name={name} size="xs" />
                ) : (
                  <span className="font-mono text-xs text-muted-foreground">{issue.assignee.slice(0, 8)}</span>
                )
              })()}
          </div>
        </div>
        {onUpdateIssue ? (
          <div className="mt-2 md:hidden" onClick={(event) => event.stopPropagation()}>
            <Select value={issue.status} onValueChange={(value) => onUpdateIssue(issue.id, value as IssueStatus)}>
              <SelectTrigger className="w-full" size="sm">
                <SelectValue>{statusLabel(issue.status)}</SelectValue>
              </SelectTrigger>
              <SelectContent align="start">
                {boardStatuses.map((status) => (
                  <SelectItem key={status} value={status}>
                    {statusLabel(status)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}
        <div className="hidden items-center gap-2 md:flex">
          <PriorityIcon priority={issue.priority} />
          {issue.assignee &&
            (() => {
              const employee = getEmployee(issue.assignee)
              const name = employee?.name
              const icon = employee?.icon
              const color = employee ? resolveEmployeeColor(employee.color) : null

              return name && icon && color ? (
                <IdentityLink color={color} employeeId={employee?.id} icon={icon} name={name} size="xs" />
              ) : (
                <span className="font-mono text-xs text-muted-foreground">{issue.assignee.slice(0, 8)}</span>
              )
            })()}
        </div>
      </div>
    </div>
  )
}

export function KanbanBoard({
  issues,
  employees,
  liveIssueIds,
  onToggleIssueSelection,
  onUpdateIssue,
  selectedIssueIds,
}: KanbanBoardProps) {
  const [activeId, setActiveId] = useState<string | null>(null)
  const hasActiveSelection = selectedIssueIds.size > 0

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }))

  const columnIssues = useMemo(() => {
    const grouped: Record<string, IssueDto[]> = {}
    for (const status of boardStatuses) {
      grouped[status] = []
    }
    for (const issue of issues) {
      if (grouped[issue.status]) {
        grouped[issue.status].push(issue)
      }
    }

    const sorted: Record<string, IssueDto[]> = {}

    for (const status of boardStatuses) {
      if (grouped[status]) {
        sorted[status] = grouped[status].toSorted((a, b) => (dayjs(a.updated_at).diff(dayjs(b.updated_at)) < 0 ? 1 : -1))
      }
    }

    return sorted
  }, [issues])

  const activeIssue = useMemo(() => (activeId ? issues.find((i) => i.id === activeId) : null), [activeId, issues])

  function handleDragStart(event: DragStartEvent) {
    setActiveId(event.active.id as string)
  }

  function handleDragEnd(event: DragEndEvent) {
    setActiveId(null)
    const { active, over } = event
    if (!over) return

    const issueId = active.id as string
    const issue = issues.find((i) => i.id === issueId)
    if (!issue) return

    let targetStatus: string | null = null

    if (boardStatuses.includes(over.id as string)) {
      targetStatus = over.id as string
    } else {
      const targetIssue = issues.find((i) => i.id === over.id)
      if (targetIssue) {
        targetStatus = targetIssue.status
      }
    }

    if (targetStatus && targetStatus !== issue.status) {
      onUpdateIssue(issueId, targetStatus as IssueStatus)
    }
  }

  function handleDragOver(_event: DragOverEvent) {}

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd} onDragOver={handleDragOver} onDragStart={handleDragStart}>
      <div className="space-y-4 px-3 pb-4 md:hidden">
        {boardStatuses.map((status) => (
          <KanbanColumn
            key={status}
            employees={employees}
            hasActiveSelection={hasActiveSelection}
            issues={columnIssues[status] ?? []}
            liveIssueIds={liveIssueIds}
            selectedIssueIds={selectedIssueIds}
            status={status}
            onToggleIssueSelection={onToggleIssueSelection}
            onUpdateIssue={onUpdateIssue}
          />
        ))}
      </div>
      <div className="hidden min-w-0 overflow-x-auto px-3 pb-4 md:block md:px-5">
        <div className="flex min-w-max gap-3">
          {boardStatuses.map((status) => (
            <KanbanColumn
              key={status}
              employees={employees}
              hasActiveSelection={hasActiveSelection}
              issues={columnIssues[status] ?? []}
              liveIssueIds={liveIssueIds}
              selectedIssueIds={selectedIssueIds}
              status={status}
              onToggleIssueSelection={onToggleIssueSelection}
              onUpdateIssue={onUpdateIssue}
            />
          ))}
        </div>
      </div>
      <DragOverlay>{activeIssue ? <KanbanCard isOverlay employees={employees} issue={activeIssue} /> : null}</DragOverlay>
    </DndContext>
  )
}