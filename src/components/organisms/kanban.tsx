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
import { useMemo, useState } from 'react'
import type { Employee } from '@/bindings/Employee'
import type { IssueDto } from '@/bindings/IssueDto'
import { IdentityLink } from '../molecules/indentity'
import { PriorityIcon } from '../molecules/priority-icon'
import { type ColorVariant, colors, fallbackColor } from '../ui/colors'
import { StatusIcon } from './status-icon'

const boardStatuses = ['backlog', 'todo', 'in_progress', 'blocked', 'done', 'cancelled']

function statusLabel(status: string): string {
  return status.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

interface KanbanBoardProps {
  issues: IssueDto[]
  employees?: Employee[]
  liveIssueIds?: Set<string>
  onUpdateIssue: (id: string, data: Record<string, unknown>) => void
}

function resolveEmployeeColor(color: string): ColorVariant {
  return colors.some((entry) => entry.color === color) ? (color as ColorVariant) : fallbackColor
}

function KanbanColumn({
  status,
  issues,
  employees,
  liveIssueIds,
}: {
  status: string
  issues: IssueDto[]
  employees?: Employee[]
  liveIssueIds?: Set<string>
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status })

  return (
    <div className="flex flex-col min-w-[260px] w-[260px] shrink-0">
      <div className="flex items-center gap-2 px-2 py-2 mb-1">
        <StatusIcon status={status} />
        <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          {statusLabel(status)}
        </span>
        <span className="text-xs text-muted-foreground/60 ml-auto tabular-nums">{issues.length}</span>
      </div>
      <div
        ref={setNodeRef}
        className={`flex-1 min-h-[120px] rounded-md p-1 space-y-1 transition-colors ${
          isOver ? 'bg-accent/40' : 'bg-muted/20'
        }`}
      >
        <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
          {issues.map((issue) => (
            <KanbanCard key={issue.id} employees={employees} isLive={liveIssueIds?.has(issue.id)} issue={issue} />
          ))}
        </SortableContext>
      </div>
    </div>
  )
}

/* ── Draggable Card ── */

function KanbanCard({
  issue,
  employees,
  isLive,
  isOverlay,
}: {
  issue: IssueDto
  employees?: Employee[]
  isLive?: boolean
  isOverlay?: boolean
}) {
  const { navigate } = useRouter()
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    data: { issue },
    id: issue.id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  const getEmployee = (id: string | null) => {
    if (!id || !employees) return null
    return employees.find((a) => a.id === id)
  }

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (isDragging) e.preventDefault()
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
      className={`rounded-md border bg-card p-2.5 cursor-grab active:cursor-grabbing transition-shadow ${
        isDragging && !isOverlay ? 'opacity-30' : ''
      } ${isOverlay ? 'shadow-lg ring-1 ring-primary/20' : 'hover:shadow-sm'}`}
    >
      <div className="block no-underline text-inherit" onClick={handleClick}>
        <div className="flex items-start gap-1.5 mb-1.5">
          <span className="text-xs text-muted-foreground font-mono shrink-0">
            {issue.identifier ?? issue.id.slice(0, 8)}
          </span>
          {isLive && (
            <span className="relative flex h-2 w-2 shrink-0 mt-0.5">
              <span className="animate-pulse absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500" />
            </span>
          )}
        </div>
        <p className="text-sm leading-snug line-clamp-2 mb-2">{issue.title}</p>
        <div className="flex items-center gap-2">
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
                <span className="text-xs text-muted-foreground font-mono">{issue.assignee.slice(0, 8)}</span>
              )
            })()}
        </div>
      </div>
    </div>
  )
}

/* ── Main Board ── */

export function KanbanBoard({ issues, employees, liveIssueIds, onUpdateIssue }: KanbanBoardProps) {
  const [activeId, setActiveId] = useState<string | null>(null)

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
        sorted[status] = grouped[status].toSorted((a, b) =>
          dayjs(a.updated_at).diff(dayjs(b.updated_at)) < 0 ? 1 : -1,
        )
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

    // Determine target status: the "over" could be a column id (status string)
    // or another card's id. Find which column the "over" belongs to.
    let targetStatus: string | null = null

    if (boardStatuses.includes(over.id as string)) {
      targetStatus = over.id as string
    } else {
      // It's a card - find which column it's in
      const targetIssue = issues.find((i) => i.id === over.id)
      if (targetIssue) {
        targetStatus = targetIssue.status
      }
    }

    if (targetStatus && targetStatus !== issue.status) {
      onUpdateIssue(issueId, { status: targetStatus })
    }
  }

  function handleDragOver(_event: DragOverEvent) {
    // Could be used for visual feedback; keeping simple for now
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd} onDragOver={handleDragOver} onDragStart={handleDragStart}>
      <div className="min-w-0 overflow-x-auto px-3 pb-4 md:px-5">
        <div className="flex min-w-max gap-3">
          {boardStatuses.map((status) => (
            <KanbanColumn
              key={status}
              employees={employees}
              issues={columnIssues[status] ?? []}
              liveIssueIds={liveIssueIds}
              status={status}
            />
          ))}
        </div>
      </div>
      <DragOverlay>
        {activeIssue ? <KanbanCard isOverlay employees={employees} issue={activeIssue} /> : null}
      </DragOverlay>
    </DndContext>
  )
}
